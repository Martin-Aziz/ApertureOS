import type {
  AuditLogEntry,
  CreateProjectPayload,
  LoginResponse,
  Project,
  RemoveBackgroundResponse
} from '../utils/contracts';

export class ApiClientError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

const DEFAULT_BACKEND_URL = 'http://localhost:8080';

export function normalizeApiPath(path: string): string {
  if (path.startsWith('/')) {
    return path;
  }

  return `/${path}`;
}

async function requestJson<T>(
  path: string,
  options: RequestInit = {},
  accessToken?: string
): Promise<T> {
  const baseUrl = import.meta.env.VITE_BACKEND_URL ?? DEFAULT_BACKEND_URL;
  const headers = new Headers(options.headers);
  headers.set('content-type', 'application/json');

  if (accessToken) {
    headers.set('authorization', `Bearer ${accessToken}`);
  }

  const response = await fetch(`${baseUrl}${normalizeApiPath(path)}`, {
    ...options,
    headers
  });

  if (!response.ok) {
    let errorMessage = `Request failed with status ${response.status}`;
    let errorCode: string | undefined;

    try {
      const payload = (await response.json()) as {
        error?: { message?: string; code?: string };
      };
      errorMessage = payload.error?.message ?? errorMessage;
      errorCode = payload.error?.code;
    } catch {
      // Non-JSON error responses use the default message.
    }

    throw new ApiClientError(errorMessage, response.status, errorCode);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export const apiClient = {
  login(email: string, password: string): Promise<LoginResponse> {
    return requestJson<LoginResponse>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password })
    });
  },

  refresh(refreshToken: string): Promise<{ access_token: string; refresh_token: string }> {
    return requestJson('/api/v1/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken })
    });
  },

  listProjects(accessToken: string): Promise<Project[]> {
    return requestJson<Project[]>('/api/v1/projects', { method: 'GET' }, accessToken);
  },

  createProject(accessToken: string, payload: CreateProjectPayload): Promise<Project> {
    return requestJson<Project>(
      '/api/v1/projects',
      {
        method: 'POST',
        body: JSON.stringify(payload)
      },
      accessToken
    );
  },

  deleteProject(accessToken: string, projectId: string): Promise<void> {
    return requestJson<void>(
      `/api/v1/projects/${projectId}`,
      {
        method: 'DELETE'
      },
      accessToken
    );
  },

  listAuditLogs(accessToken: string): Promise<AuditLogEntry[]> {
    return requestJson<AuditLogEntry[]>('/api/v1/admin/audit-logs', { method: 'GET' }, accessToken);
  },

  removeBackground(imageBase64: string): Promise<RemoveBackgroundResponse> {
    return requestJson<RemoveBackgroundResponse>('/api/v1/ai/remove-background', {
      method: 'POST',
      body: JSON.stringify({ image_base64: imageBase64 })
    });
  }
};
