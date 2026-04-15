import { writable } from 'svelte/store';

import { apiClient } from '../services/api-client';
import type { Project } from '../utils/contracts';

interface ProjectState {
  items: Project[];
  isLoading: boolean;
  error: string | null;
}

const initialState: ProjectState = {
  items: [],
  isLoading: false,
  error: null
};

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectState>(initialState);

  return {
    subscribe,

    reset(): void {
      set(initialState);
    },

    async load(accessToken: string): Promise<void> {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const projects = await apiClient.listProjects(accessToken);
        set({ items: projects, isLoading: false, error: null });
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to load projects';
        update((state) => ({ ...state, isLoading: false, error: message }));
      }
    },

    async create(accessToken: string, name: string, description: string): Promise<void> {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const project = await apiClient.createProject(accessToken, { name, description });
        update((state) => ({
          items: [project, ...state.items],
          isLoading: false,
          error: null
        }));
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to create project';
        update((state) => ({ ...state, isLoading: false, error: message }));
      }
    },

    async remove(accessToken: string, projectId: string): Promise<void> {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        await apiClient.deleteProject(accessToken, projectId);
        update((state) => ({
          ...state,
          items: state.items.filter((item) => item.id !== projectId),
          isLoading: false,
          error: null
        }));
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to delete project';
        update((state) => ({ ...state, isLoading: false, error: message }));
      }
    }
  };
}

export const projectStore = createProjectStore();
