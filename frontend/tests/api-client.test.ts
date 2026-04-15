import { describe, expect, it } from 'vitest';

import { normalizeApiPath } from '../src/services/api-client';

describe('normalizeApiPath', () => {
  it('should keep paths that already start with slash', () => {
    expect(normalizeApiPath('/api/v1/projects')).toBe('/api/v1/projects');
  });

  it('should prepend slash when path is missing one', () => {
    expect(normalizeApiPath('api/v1/projects')).toBe('/api/v1/projects');
  });
});
