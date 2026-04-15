import { z } from 'zod';

import {
  MAX_PROJECT_DESCRIPTION_LENGTH,
  MAX_PROJECT_NAME_LENGTH,
  MIN_PROJECT_NAME_LENGTH
} from '../constants/limits';

export const createProjectSchema = z
  .object({
    name: z
      .string()
      .trim()
      .min(MIN_PROJECT_NAME_LENGTH)
      .max(MAX_PROJECT_NAME_LENGTH),
    description: z
      .string()
      .trim()
      .max(MAX_PROJECT_DESCRIPTION_LENGTH)
      .optional()
  })
  .strict();

export type CreateProjectInput = z.infer<typeof createProjectSchema>;
