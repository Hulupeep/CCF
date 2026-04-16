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
    // Excluded per #122 story 2c.1 — no test imports these. Previously emitted
    // TS errors during coverage-collection compilation that were log pollution
    // only (no test suite depended on them).  AnimatedPersonalitySlider appears
    // to be orphaned dead code (zero repo-wide imports) — flagged for a future
    // audit.  EmailAccounts is transitively-imported by VoiceDashboard which
    // itself has no test, so the tree is unreachable from any test.
    '!web/src/components/AnimatedPersonalitySlider.tsx',
    '!web/src/components/voice/EmailAccounts.tsx',
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
        // Explicit `types` needed here because ts-jest's inline tsconfig REPLACES
        // rather than extends, so the usual "all @types auto-included" behaviour
        // doesn't apply. List every @types/* we rely on during jest compilation.
        types: ['jest', 'node', 'dom-speech-recognition', 'styled-jsx'],
      },
    },
  },
  setupFilesAfterEnv: ['<rootDir>/tests/setup.ts'],
};
