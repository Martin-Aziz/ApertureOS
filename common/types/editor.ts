export type UserRole = 'admin' | 'editor';

export interface AuthUser {
  userId: string;
  email: string;
  role: UserRole;
}

export interface Project {
  id: string;
  ownerId: string;
  name: string;
  description: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}
