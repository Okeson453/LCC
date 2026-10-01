module.exports = {
  ci: {
    collect: {
      url: [
        'http://localhost:3000/today',
        'http://localhost:3000/approvals',
        'http://localhost:3000/content',
        'http://localhost:3000/analytics',
      ],
      numberOfRuns: 3,
      settings: {
        preset: 'desktop',
      },
    },
    assert: {
      assertions: {
        'categories:performance': ['error', { minScore: 0.9 }],
        'categories:accessibility': ['error', { minScore: 0.95 }],
        'categories:best-practices': ['error', { minScore: 0.9 }],
        'categories:seo': ['warn', { minScore: 0.85 }],
      },
    },
  },
};
