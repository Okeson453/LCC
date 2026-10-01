/**
 * Root ESLint configuration for the OKESON-LCC frontend monorepo.
 * Extends @lcc/eslint-config (workspace package, sibling to this file) and
 * enables project-specific rule overrides for non-TS files.
 */
module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: 'module',
    ecmaFeatures: { jsx: true },
  },
  env: {
    browser: true,
    es2022: true,
    node: true,
  },
  ignorePatterns: [
    'node_modules',
    '.next',
    'dist',
    'build',
    'coverage',
    '*.config.js',
    '*.config.mjs',
    '*.config.ts',
  ],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended',
    'plugin:jsx-a11y/recommended',
    'plugin:import/recommended',
    'prettier',
  ],
  settings: {
    react: { version: 'detect' },
    'import/resolver': {
      typescript: { project: ['./tsconfig.base.json', './apps/*/tsconfig.json', './packages/*/tsconfig.json'] },
      node: { extensions: ['.js', '.ts', '.tsx'] },
    },
  },
  rules: {
    'react/react-in-jsx-scope': 'off',
    'react/prop-types': 'off',
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
    '@typescript-eslint/consistent-type-imports': ['warn', { prefer: 'type-imports' }],
    '@typescript-eslint/no-explicit-any': 'error',
    'no-restricted-imports': [
      'error',
      {
        patterns: [
          {
            group: ['../../../packages/*', '../../../../packages/*'],
            message: 'Use @lcc/* package aliases instead of relative cross-package imports.',
          },
        ],
      },
    ],
    'no-restricted-syntax': [
      'error',
      {
        selector: "CallExpression[callee.object.name='console'][callee.property.name=/^(log|debug|info)$/]",
        message: 'Use the structured logger from @lcc/test-utils/mocks/logger; do not log PII or tokens to console.',
      },
    ],
  },
  overrides: [
    {
      files: ['*.test.ts', '*.test.tsx', '**/tests/**'],
      rules: {
        '@typescript-eslint/no-explicit-any': 'off',
        'no-restricted-syntax': 'off',
      },
    },
    {
      files: ['apps/browser-extension/src/**', 'apps/browser-extension/src/**/*.ts'],
      rules: {
        'no-restricted-syntax': [
          'error',
          {
            selector: "MemberExpression[property.name='chrome'][property.name!='runtime']",
            message: 'Only chrome.runtime APIs are allowed; do not call chrome.tabs, chrome.storage, etc. outside background.',
          },
        ],
      },
    },
  ],
};
