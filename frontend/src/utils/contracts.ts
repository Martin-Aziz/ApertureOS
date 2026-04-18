export type UserRole = 'admin' | 'editor';

export interface AuthUser {
  user_id: string;
  email: string;
  role: UserRole;
}

export interface TokenPair {
  access_token: string;
  refresh_token: string;
  expires_in_seconds: number;
}

export interface LoginResponse {
  tokens: TokenPair;
  user: AuthUser;
}

export interface Project {
  id: string;
  owner_id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

export interface CreateProjectPayload {
  name: string;
  description?: string;
}

export interface AuditLogEntry {
  id: string;
  actor_id: string;
  action: string;
  resource_type: string;
  resource_id: string | null;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface RemoveBackgroundResponse {
  image_base64: string;
  provider: string;
  processing_ms: number;
}

export interface ApiErrorEnvelope {
  error: {
    code: string;
    message: string;
  };
}
