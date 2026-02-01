module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'jsdom', // Changed to jsdom for browser APIs (window, document)
  roots: ['<rootDir>/tests', '<rootDir>/web/src'],
  testMatch: [
    '**/__tests__/**/*.+(ts|tsx|js)',
    '**/?(*.)+(spec|test).+(ts|tsx|js)',
  ],
  transform: {
    '^.+\\.(ts|tsx)$': 'ts-jest',
  },
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  collectCoverageFrom: [
    'web/src/**/*.{ts,tsx}',
    '!web/src/**/*.d.ts',
    '!web/src/**/__tests__/**',
  ],
  moduleNameMapper: {
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
    '^@/(.*)$': '<rootDir>/web/src/$1',
  },
  globals: {
    'ts-jest': {
      tsconfig: {
        jsx: 'react',
        esModuleInterop: true,
        allowSyntheticDefaultImports: true,
        lib: ['ES2015', 'DOM'],
      },
    },
  },
  setupFilesAfterEnv: ['<rootDir>/tests/setup.ts'],
};
