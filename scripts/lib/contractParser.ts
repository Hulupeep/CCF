/**
 * Contract Parser - Extract journey contracts from YAML files
 * Part of Issue #81: Journey Coverage Report Tool
 */

import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'yaml';

export interface JourneyContract {
  id: string;
  name: string;
  file: string;
  status: 'active' | 'draft' | 'deprecated';
  type: 'e2e';
  dod_criticality: 'critical' | 'important' | 'future';
  dod_status: 'passing' | 'failing' | 'not_tested';
  covers_reqs: string[];
  summary: string;
  e2e_test?: string;
}

export interface FeatureContract {
  id: string;
  file: string;
  status: string;
  covers_reqs: string[];
  summary: string;
}

export interface ContractIndex {
  metadata: {
    project: string;
    version: number;
    created: string;
    total_contracts: number;
    total_requirements: string;
    total_journeys: number;
  };
  definition_of_done: {
    critical_journeys: string[];
    important_journeys: string[];
    future_journeys: string[];
    release_gate: string;
  };
  contracts: Array<JourneyContract | FeatureContract>;
  requirements_coverage: Record<string, string | string[]>;
  implementation_status: {
    completed: string[];
    in_progress: string[];
    not_started: string[];
  };
  uncovered_requirements: string[];
}

export class ContractParser {
  private contractsDir: string;

  constructor(contractsDir: string = 'docs/contracts') {
    this.contractsDir = contractsDir;
  }

  /**
   * Parse the main CONTRACT_INDEX.yml file
   */
  parseContractIndex(): ContractIndex {
    const indexPath = path.join(this.contractsDir, 'CONTRACT_INDEX.yml');

    if (!fs.existsSync(indexPath)) {
      throw new Error(`Contract index not found at ${indexPath}`);
    }

    const content = fs.readFileSync(indexPath, 'utf-8');
    const parsed = yaml.parse(content);

    return parsed as ContractIndex;
  }

  /**
   * Extract all journey contracts from the index
   */
  extractJourneyContracts(): JourneyContract[] {
    const index = this.parseContractIndex();

    const journeys = index.contracts.filter(
      (contract): contract is JourneyContract =>
        contract.id.startsWith('J-') && 'type' in contract && contract.type === 'e2e'
    );

    return journeys;
  }

  /**
   * Parse a specific feature contract YAML file
   */
  parseFeatureContract(filename: string): any {
    const filepath = path.join(this.contractsDir, filename);

    if (!fs.existsSync(filepath)) {
      throw new Error(`Contract file not found: ${filepath}`);
    }

    const content = fs.readFileSync(filepath, 'utf-8');
    return yaml.parse(content);
  }

  /**
   * Get all journey contracts grouped by criticality
   */
  getJourneysByCriticality(): {
    critical: JourneyContract[];
    important: JourneyContract[];
    future: JourneyContract[];
  } {
    const journeys = this.extractJourneyContracts();

    return {
      critical: journeys.filter(j => j.dod_criticality === 'critical'),
      important: journeys.filter(j => j.dod_criticality === 'important'),
      future: journeys.filter(j => j.dod_criticality === 'future'),
    };
  }

  /**
   * Get DOD requirements from index
   */
  getDODRequirements() {
    const index = this.parseContractIndex();
    return index.definition_of_done;
  }

  /**
   * Get requirements coverage mapping
   */
  getRequirementsCoverage() {
    const index = this.parseContractIndex();
    return index.requirements_coverage;
  }

  /**
   * Get implementation status
   */
  getImplementationStatus() {
    const index = this.parseContractIndex();
    return index.implementation_status;
  }
}
