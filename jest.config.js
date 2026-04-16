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
    // Excluded per #122 story 2b — these files are not part of the jest coverage
    // surface. vite.config.ts is a build-time config (imports @vitejs/plugin-react
    // which ts-jest cannot resolve); telegram-bot.ts is a node-only CLI entry
    // point; examples/ is demo code not exercised by any test.
    '!web/src/vite.config.ts',
    '!web/src/telegram-bot.ts',
    '!web/src/examples/**',
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
