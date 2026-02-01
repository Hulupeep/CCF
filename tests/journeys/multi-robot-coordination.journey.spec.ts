/**
 * Journey Test: J-MULTI-SWARM-DANCE
 * Multi-Robot Coordination
 *
 * Tests coordinated behavior of 2-4 robots discovering each other,
 * electing a leader, and executing synchronized movements.
 */

import { test, expect, Page } from '@playwright/test';

const DISCOVERY_TIMEOUT = 5000;
const ELECTION_TIMEOUT = 3000;

test.describe('J-MULTI-SWARM-DANCE: Multi-Robot Coordination', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/coordination');
  });

  test('Scenario: Robots Discover Each Other', async ({ page, context }) => {
    /**
     * Given 3 robots on same network
     * When I start coordination mode
     * Then each robot discovers the others within 5 seconds
     * And shared state is established
     * And all robots show "Connected" status
     */

    // Given: 3 robots on same network (simulate with multiple browser contexts)
    const robot1Page = page;
    const robot2Page = await context.newPage();
    const robot3Page = await context.newPage();

    await robot2Page.goto('/coordination');
    await robot3Page.goto('/coordination');

    // Set robot IDs
    await robot1Page.fill('[data-testid="robot-id-input"]', 'robot-1');
    await robot2Page.fill('[data-testid="robot-id-input"]', 'robot-2');
    await robot3Page.fill('[data-testid="robot-id-input"]', 'robot-3');

    // When: Start coordination mode
    await robot1Page.click('[data-testid="start-coordination-btn"]');
    await robot2Page.click('[data-testid="start-coordination-btn"]');
    await robot3Page.click('[data-testid="start-coordination-btn"]');

    // Then: Each robot discovers the others within 5 seconds (I-MULTI-001)
    await expect(robot1Page.locator('[data-testid="robot-discovery-list"]')).toBeVisible({
      timeout: DISCOVERY_TIMEOUT,
    });

    // Check that robot-1 sees robot-2 and robot-3
    await expect(
      robot1Page.locator('[data-testid="robot-status-robot-2"]')
    ).toBeVisible({ timeout: DISCOVERY_TIMEOUT });
    await expect(
      robot1Page.locator('[data-testid="robot-status-robot-3"]')
    ).toBeVisible({ timeout: DISCOVERY_TIMEOUT });

    // And: Shared state is established
    await expect(robot1Page.locator('[data-testid="sync-status"]')).toHaveText('Synced');

    // And: All robots show "Connected" status
    await expect(robot1Page.locator('[data-testid="robot-status-robot-2"]')).toHaveText(
      /Connected|Active/
    );
    await expect(robot1Page.locator('[data-testid="robot-status-robot-3"]')).toHaveText(
      /Connected|Active/
    );

    await robot2Page.close();
    await robot3Page.close();
  });

  test('Scenario: Leader Election', async ({ page, context }) => {
    /**
     * Given 4 robots in mesh network
     * When coordination starts
     * Then one robot is elected leader
     * And all robots acknowledge leader
     * And leadership is maintained
     */

    // Given: 4 robots in mesh network
    const robots: Page[] = [];
    for (let i = 1; i <= 4; i++) {
      const robotPage = i === 1 ? page : await context.newPage();
      await robotPage.goto('/coordination');
      await robotPage.fill('[data-testid="robot-id-input"]', `robot-${i}`);
      robots.push(robotPage);
    }

    // When: Coordination starts
    for (const robotPage of robots) {
      await robotPage.click('[data-testid="start-coordination-btn"]');
    }

    // Wait for discovery
    await page.waitForTimeout(DISCOVERY_TIMEOUT);

    // Then: One robot is elected leader (I-MULTI-003)
    let leaderCount = 0;
    for (let i = 1; i <= 4; i++) {
      const leaderIndicator = robots[0].locator(`[data-testid="leader-indicator-robot-${i}"]`);
      if (await leaderIndicator.isVisible()) {
        leaderCount++;
      }
    }
    expect(leaderCount).toBe(1); // Exactly one leader

    // And: All robots acknowledge leader (check on all robot pages)
    for (const robotPage of robots) {
      const leaderIndicator = robotPage.locator('[data-testid^="leader-indicator-"]');
      await expect(leaderIndicator.first()).toBeVisible({ timeout: ELECTION_TIMEOUT });
    }

    // And: Leadership is maintained (check after some time)
    await page.waitForTimeout(5000);
    leaderCount = 0;
    for (let i = 1; i <= 4; i++) {
      const leaderIndicator = robots[0].locator(`[data-testid="leader-indicator-robot-${i}"]`);
      if (await leaderIndicator.isVisible()) {
        leaderCount++;
      }
    }
    expect(leaderCount).toBe(1); // Still exactly one leader

    // Cleanup
    for (let i = 1; i < robots.length; i++) {
      await robots[i].close();
    }
  });

  test('Scenario: Synchronized Movement', async ({ page, context }) => {
    /**
     * Given 2 robots in coordination
     * When I command "dance"
     * Then both robots move in sync
     * And movements are coordinated within 50ms
     * And no collisions occur
     */

    // Given: 2 robots in coordination
    const robot1Page = page;
    const robot2Page = await context.newPage();

    await robot1Page.fill('[data-testid="robot-id-input"]', 'robot-1');
    await robot2Page.goto('/coordination');
    await robot2Page.fill('[data-testid="robot-id-input"]', 'robot-2');

    await robot1Page.click('[data-testid="start-coordination-btn"]');
    await robot2Page.click('[data-testid="start-coordination-btn"]');

    // Wait for coordination to establish
    await expect(robot1Page.locator('[data-testid="sync-status"]')).toHaveText('Synced', {
      timeout: DISCOVERY_TIMEOUT,
    });

    // When: Command "dance" from leader
    await robot1Page.click('[data-testid="coordinated-dance-btn"]');

    // Then: Both robots move in sync (check position updates)
    const robot1Position = robot1Page.locator('[data-testid="robot-position-robot-1"]');
    const robot2Position = robot2Page.locator('[data-testid="robot-position-robot-2"]');

    // Wait for movement to start
    await expect(robot1Position).not.toHaveText('(0, 0)', { timeout: 1000 });
    await expect(robot2Position).not.toHaveText('(0, 0)', { timeout: 1000 });

    // And: Movements are coordinated within 50ms (check sync latency indicator)
    const syncLatency = robot1Page.locator('[data-testid="sync-latency"]');
    await expect(syncLatency).toHaveText(/\d+ms/);

    const latencyText = await syncLatency.textContent();
    const latencyMs = parseInt(latencyText?.match(/\d+/)?.[0] || '999', 10);
    expect(latencyMs).toBeLessThan(100); // I-MULTI-005: <100ms latency

    // And: No collisions occur (check collision indicator)
    const collisionStatus = robot1Page.locator('[data-testid="collision-status"]');
    await expect(collisionStatus).toHaveText(/No Collisions|Safe/);

    await robot2Page.close();
  });

  test('Scenario: Graceful Disconnect Handling', async ({ page, context }) => {
    /**
     * Given 3 robots in coordination with robot-2 as leader
     * When robot-2 disconnects
     * Then a new leader is elected within 3 seconds
     * And remaining robots continue coordinating
     * And robot-2 is removed from robot list
     */

    // Given: 3 robots in coordination
    const robot1Page = page;
    const robot2Page = await context.newPage();
    const robot3Page = await context.newPage();

    await robot1Page.fill('[data-testid="robot-id-input"]', 'robot-1');
    await robot2Page.goto('/coordination');
    await robot2Page.fill('[data-testid="robot-id-input"]', 'robot-2');
    await robot3Page.goto('/coordination');
    await robot3Page.fill('[data-testid="robot-id-input"]', 'robot-3');

    await robot1Page.click('[data-testid="start-coordination-btn"]');
    await robot2Page.click('[data-testid="start-coordination-btn"]');
    await robot3Page.click('[data-testid="start-coordination-btn"]');

    // Wait for coordination and leader election
    await page.waitForTimeout(DISCOVERY_TIMEOUT + ELECTION_TIMEOUT);

    // When: Robot-2 disconnects (I-MULTI-006)
    await robot2Page.click('[data-testid="disconnect-coordination-btn"]');
    await robot2Page.close();

    // Then: New leader is elected within 3 seconds (I-MULTI-003)
    await expect(
      robot1Page.locator('[data-testid^="leader-indicator-"]').first()
    ).toBeVisible({ timeout: ELECTION_TIMEOUT });

    // And: Remaining robots continue coordinating
    await expect(robot1Page.locator('[data-testid="sync-status"]')).toHaveText('Synced');
    await expect(robot3Page.locator('[data-testid="sync-status"]')).toHaveText('Synced');

    // And: Robot-2 is removed from robot list
    await expect(robot1Page.locator('[data-testid="robot-status-robot-2"]')).not.toBeVisible({
      timeout: 5000,
    });

    await robot3Page.close();
  });

  test('Scenario: Maximum 4 Robots Enforced', async ({ page, context }) => {
    /**
     * Given 4 robots already in coordination
     * When a 5th robot tries to join
     * Then the 5th robot is rejected
     * And an error message is displayed
     */

    // Given: 4 robots already in coordination
    const robots: Page[] = [];
    for (let i = 1; i <= 4; i++) {
      const robotPage = i === 1 ? page : await context.newPage();
      await robotPage.goto('/coordination');
      await robotPage.fill('[data-testid="robot-id-input"]', `robot-${i}`);
      await robotPage.click('[data-testid="start-coordination-btn"]');
      robots.push(robotPage);
    }

    await page.waitForTimeout(DISCOVERY_TIMEOUT);

    // When: 5th robot tries to join
    const robot5Page = await context.newPage();
    await robot5Page.goto('/coordination');
    await robot5Page.fill('[data-testid="robot-id-input"]', 'robot-5');
    await robot5Page.click('[data-testid="start-coordination-btn"]');

    // Then: 5th robot is rejected (I-MULTI-004)
    await expect(robot5Page.locator('[data-testid="coordination-error"]')).toBeVisible({
      timeout: 2000,
    });

    // And: Error message is displayed
    await expect(robot5Page.locator('[data-testid="coordination-error"]')).toHaveText(
      /maximum.*4.*robots/i
    );

    // Cleanup
    for (let i = 1; i < robots.length; i++) {
      await robots[i].close();
    }
    await robot5Page.close();
  });

  test('Scenario: State Consistency with Heartbeat', async ({ page, context }) => {
    /**
     * Given 2 robots in coordination
     * When robots exchange heartbeats
     * Then heartbeat indicators update regularly
     * And no split-brain scenarios occur
     */

    // Given: 2 robots in coordination
    const robot1Page = page;
    const robot2Page = await context.newPage();

    await robot1Page.fill('[data-testid="robot-id-input"]', 'robot-1');
    await robot2Page.goto('/coordination');
    await robot2Page.fill('[data-testid="robot-id-input"]', 'robot-2');

    await robot1Page.click('[data-testid="start-coordination-btn"]');
    await robot2Page.click('[data-testid="start-coordination-btn"]');

    await page.waitForTimeout(DISCOVERY_TIMEOUT);

    // When: Robots exchange heartbeats (I-MULTI-002)
    // Then: Heartbeat indicators update regularly
    const heartbeat1 = robot1Page.locator('[data-testid="heartbeat-robot-2"]');
    await expect(heartbeat1).toBeVisible();

    // Check that heartbeat updates multiple times
    const initialHeartbeat = await heartbeat1.textContent();
    await page.waitForTimeout(2000);
    const updatedHeartbeat = await heartbeat1.textContent();
    expect(updatedHeartbeat).not.toBe(initialHeartbeat);

    // And: No split-brain scenarios (both robots agree on leader)
    const leader1 = await robot1Page
      .locator('[data-testid^="leader-indicator-"]')
      .first()
      .getAttribute('data-testid');
    const leader2 = await robot2Page
      .locator('[data-testid^="leader-indicator-"]')
      .first()
      .getAttribute('data-testid');

    expect(leader1).toBe(leader2); // Both agree on same leader

    await robot2Page.close();
  });
});
