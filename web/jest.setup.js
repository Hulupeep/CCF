// Jest setup file for testing
import '@testing-library/jest-dom';

// Mock window.devicePixelRatio
Object.defineProperty(window, 'devicePixelRatio', {
  writable: true,
  configurable: true,
  value: 1,
});

// Mock HTMLCanvasElement.getContext
HTMLCanvasElement.prototype.getContext = jest.fn(() => ({
  fillStyle: '',
  strokeStyle: '',
  lineWidth: 0,
  font: '',
  textAlign: '',
  textBaseline: '',
  clearRect: jest.fn(),
  fillRect: jest.fn(),
  strokeRect: jest.fn(),
  fillText: jest.fn(),
  moveTo: jest.fn(),
  lineTo: jest.fn(),
  beginPath: jest.fn(),
  closePath: jest.fn(),
  stroke: jest.fn(),
  fill: jest.fn(),
  arc: jest.fn(),
  createRadialGradient: jest.fn(() => ({
    addColorStop: jest.fn(),
  })),
  scale: jest.fn(),
  save: jest.fn(),
  restore: jest.fn(),
}));

// Mock requestAnimationFrame
global.requestAnimationFrame = (cb) => {
  return setTimeout(cb, 16);
};

global.cancelAnimationFrame = (id) => {
  clearTimeout(id);
};
