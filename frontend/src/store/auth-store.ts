import { browser } from '$app/environment';
import { writable } from 'svelte/store';

import { apiClient } from '../services/api-client';
import type { AuthUser } from '../utils/contracts';

interface AuthState {
  user: AuthUser | null;
  accessToken: string | null;
  refreshToken: string | null;
  expiresAt: number | null;
  isLoading: boolean;
  error: string | null;
}

const STORAGE_KEY = 'pixelforge-auth';

const initialState: AuthState = {
  user: null,
  accessToken: null,
  refreshToken: null,
  expiresAt: null,
  isLoading: false,
  error: null
};

function loadStateFromStorage(): AuthState {
  if (!browser) {
    return initialState;
  }

  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return initialState;
  }

  try {
    const parsed = JSON.parse(raw) as AuthState;
    return {
      ...initialState,
      ...parsed,
      isLoading: false,
      error: null
    };
  } catch {
    localStorage.removeItem(STORAGE_KEY);
    return initialState;
  }
}

function persistState(state: AuthState): void {
  if (!browser) {
    return;
  }

  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function createAuthStore() {
  const { subscribe, update, set } = writable<AuthState>(loadStateFromStorage());

  return {
    subscribe,

    async login(email: string, password: string): Promise<void> {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const response = await apiClient.login(email, password);
        const nextState: AuthState = {
          user: response.user,
          accessToken: response.tokens.access_token,
          refreshToken: response.tokens.refresh_token,
          expiresAt: Date.now() + response.tokens.expires_in_seconds * 1000,
          isLoading: false,
          error: null
        };

        set(nextState);
        persistState(nextState);
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Login failed';
        update((state) => ({ ...state, isLoading: false, error: message }));
      }
    },

    logout(): void {
      set(initialState);
      if (browser) {
        localStorage.removeItem(STORAGE_KEY);
      }
    }
  };
}

export const authStore = createAuthStore();
