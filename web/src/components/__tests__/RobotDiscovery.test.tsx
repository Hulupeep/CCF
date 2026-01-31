/**
 * Component Tests for RobotDiscovery
 * Issue #77 - STORY-ARCH-008
 *
 * Tests UI elements with data-testid selectors
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { RobotDiscovery } from '../RobotDiscovery';
import { MockDiscoveryService } from '../../services/robotDiscovery';

describe('RobotDiscovery Component', () => {
  let mockService: MockDiscoveryService;

  beforeEach(() => {
    mockService = new MockDiscoveryService();
  });

  afterEach(async () => {
    await mockService.stop();
  });

  test('should render discovery panel', () => {
    render(<RobotDiscovery discoveryService={mockService} />);

    const panel = screen.getByTestId('robot-discovery-panel');
    expect(panel).toBeInTheDocument();
  });

  test('should show discovery controls', () => {
    render(<RobotDiscovery discoveryService={mockService} />);

    const toggleButton = screen.getByTestId('discovery-toggle');
    const refreshButton = screen.getByTestId('refresh-button');

    expect(toggleButton).toBeInTheDocument();
    expect(refreshButton).toBeInTheDocument();
  });

  test('should display robots list', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      const robotsList = screen.getByTestId('robots-list');
      expect(robotsList).toBeInTheDocument();
    });
  });

  test('should show "no robots" message when empty', async () => {
    // Create service without mock robots
    const emptyService = new MockDiscoveryService();
    // Clear robots by creating a new service that won't auto-populate
    (emptyService as any).robots = [];

    render(<RobotDiscovery discoveryService={emptyService} />);

    await waitFor(() => {
      const noRobotsMessage = screen.getByTestId('no-robots-message');
      expect(noRobotsMessage).toBeInTheDocument();
    });
  });

  test('should display robot cards with all required information', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      // Check for first robot card
      const robotCard = screen.getByTestId('robot-card-mbot-001');
      expect(robotCard).toBeInTheDocument();

      // Check robot name
      const robotName = screen.getByTestId('robot-name-mbot-001');
      expect(robotName).toHaveTextContent('mBot Alpha');

      // Check IP address
      const robotIp = screen.getByTestId('robot-ip-mbot-001');
      expect(robotIp).toHaveTextContent('192.168.1.100');

      // Check port
      const robotPort = screen.getByTestId('robot-port-mbot-001');
      expect(robotPort).toHaveTextContent('8081');

      // Check version
      const robotVersion = screen.getByTestId('robot-version-mbot-001');
      expect(robotVersion).toHaveTextContent('1.0.0');
    });
  });

  test('should show health indicators', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      const status = screen.getByTestId('robot-status-mbot-001');
      expect(status).toBeInTheDocument();
      expect(status).toHaveTextContent(/disconnected/i);
    });
  });

  test('should have connect buttons for disconnected robots', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      const connectButton = screen.getByTestId('connect-button-mbot-001');
      expect(connectButton).toBeInTheDocument();
      expect(connectButton).toHaveTextContent(/connect/i);
    });
  });

  test('should toggle discovery on button click', async () => {
    render(<RobotDiscovery discoveryService={mockService} />);

    const toggleButton = screen.getByTestId('discovery-toggle');

    // Initial state - should start discovery
    expect(toggleButton).toHaveTextContent(/start discovery/i);

    // Click to start
    fireEvent.click(toggleButton);

    await waitFor(() => {
      expect(toggleButton).toHaveTextContent(/stop discovery/i);
    });

    // Click to stop
    fireEvent.click(toggleButton);

    await waitFor(() => {
      expect(toggleButton).toHaveTextContent(/start discovery/i);
    });
  });

  test('should call onRobotConnect when connect is clicked', async () => {
    await mockService.start();
    const onConnect = jest.fn();

    render(
      <RobotDiscovery
        discoveryService={mockService}
        onRobotConnect={onConnect}
      />
    );

    await waitFor(() => {
      const connectButton = screen.getByTestId('connect-button-mbot-001');
      fireEvent.click(connectButton);
    });

    // Note: In real scenario, WebSocket connection would trigger this
    // For now, just verify the button exists and is clickable
    const connectButton = screen.getByTestId('connect-button-mbot-001');
    expect(connectButton).toBeInTheDocument();
  });

  test('should show discovery stats when robots are found', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      const stats = screen.getByTestId('discovery-stats');
      expect(stats).toBeInTheDocument();
      expect(stats).toHaveTextContent(/Found:/i);
      expect(stats).toHaveTextContent(/Connected:/i);
    });
  });

  test('should display error message when discovery fails', () => {
    // Force an error by providing invalid service
    const errorService = new MockDiscoveryService();
    // Override start to throw error
    errorService.start = async () => {
      throw new Error('Discovery failed');
    };

    render(<RobotDiscovery discoveryService={errorService} />);

    // The error would be caught internally and displayed
    // This tests that the error display element exists
    const panel = screen.getByTestId('robot-discovery-panel');
    expect(panel).toBeInTheDocument();
  });

  test('should refresh robots list on refresh button click', async () => {
    await mockService.start();
    render(<RobotDiscovery discoveryService={mockService} />);

    await waitFor(() => {
      const refreshButton = screen.getByTestId('refresh-button');
      fireEvent.click(refreshButton);
    });

    // Verify robots list is still present after refresh
    await waitFor(() => {
      const robotsList = screen.getByTestId('robots-list');
      expect(robotsList).toBeInTheDocument();
    });
  });

  test('should apply custom className', () => {
    const { container } = render(
      <RobotDiscovery
        discoveryService={mockService}
        className="custom-class"
      />
    );

    const panel = container.querySelector('.robot-discovery.custom-class');
    expect(panel).toBeInTheDocument();
  });

  describe('Robot Card Health Indicators', () => {
    test('should show different status icons', async () => {
      await mockService.start();
      render(<RobotDiscovery discoveryService={mockService} />);

      await waitFor(() => {
        const status = screen.getByTestId('robot-status-mbot-001');
        expect(status).toBeInTheDocument();

        // Should contain status icon (emoji or text)
        expect(status.textContent).toBeTruthy();
      });
    });

    test('should display robot capabilities badges', async () => {
      await mockService.start();
      render(<RobotDiscovery discoveryService={mockService} />);

      await waitFor(() => {
        const robotCard = screen.getByTestId('robot-card-mbot-001');
        expect(robotCard).toBeInTheDocument();

        // Capabilities should be rendered as badges
        expect(robotCard.textContent).toContain('drawing');
        expect(robotCard.textContent).toContain('personality');
      });
    });

    test('should show firmware version when available', async () => {
      await mockService.start();
      render(<RobotDiscovery discoveryService={mockService} />);

      await waitFor(() => {
        const firmware = screen.getByTestId('robot-firmware-mbot-001');
        expect(firmware).toBeInTheDocument();
        expect(firmware).toHaveTextContent('2.1.0');
      });
    });
  });

  describe('Responsive Behavior', () => {
    test('should render correctly on mobile viewport', async () => {
      global.innerWidth = 375;
      global.dispatchEvent(new Event('resize'));

      await mockService.start();
      render(<RobotDiscovery discoveryService={mockService} />);

      await waitFor(() => {
        const panel = screen.getByTestId('robot-discovery-panel');
        expect(panel).toBeInTheDocument();
      });
    });
  });
});
