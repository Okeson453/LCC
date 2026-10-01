import type { ContentItem } from '@lcc/api-types';

export const mockContentItemDraft: ContentItem = {
  id: 'c1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  status: 'draft',
  body: 'Just shipped our new feature. Here is what it does and why it matters…',
  title: 'New feature release',
  media: [],
  variant: 'authority',
  scheduled_at: null,
  published_at: null,
  trace_id: 't1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  kb_refs: [],
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

export const mockContentItemPublished: ContentItem = {
  ...mockContentItemDraft,
  id: 'c2d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  status: 'published',
  published_at: new Date().toISOString(),
};
