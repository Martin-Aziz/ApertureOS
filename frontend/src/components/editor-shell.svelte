<script lang="ts">
  import { onMount } from 'svelte';

  import { authStore } from '../store/auth-store';
  import { projectStore } from '../store/project-store';

  let email = 'admin@pixelforge.local';
  let password = 'ChangeMe123!';
  let projectName = '';
  let projectDescription = '';

  $: isAuthenticated = Boolean($authStore.accessToken);

  onMount(async () => {
    if ($authStore.accessToken) {
      await projectStore.load($authStore.accessToken);
    }
  });

  async function login(): Promise<void> {
    await authStore.login(email, password);

    if ($authStore.accessToken) {
      await projectStore.load($authStore.accessToken);
    }
  }

  async function createProject(): Promise<void> {
    if (!$authStore.accessToken) {
      return;
    }

    await projectStore.create(
      $authStore.accessToken,
      projectName,
      projectDescription
    );

    projectName = '';
    projectDescription = '';
  }

  async function deleteProject(projectId: string): Promise<void> {
    if (!$authStore.accessToken) {
      return;
    }

    await projectStore.remove($authStore.accessToken, projectId);
  }

  function logout(): void {
    authStore.logout();
    projectStore.reset();
  }
</script>

<div class="page">
  <div class="nebula"></div>
  <main class="shell" aria-live="polite">
    <header class="hero">
      <p class="badge">PixelForge MVP Console</p>
      <h1>Browser-first image workflow starts here.</h1>
      <p>
        This shell wires auth, project lifecycle, and protected API calls so the
        editor domain can grow on a stable foundation.
      </p>
    </header>

    {#if !isAuthenticated}
      <section class="panel">
        <h2>Sign in</h2>
        <p class="hint">Use the seeded backend credentials for this scaffold.</p>

        <label for="email">Email</label>
        <input id="email" bind:value={email} autocomplete="email" />

        <label for="password">Password</label>
        <input
          id="password"
          bind:value={password}
          type="password"
          autocomplete="current-password"
        />

        {#if $authStore.error}
          <p class="error">{$authStore.error}</p>
        {/if}

        <button on:click={login} disabled={$authStore.isLoading}>
          {$authStore.isLoading ? 'Signing in...' : 'Sign in'}
        </button>
      </section>
    {:else}
      <section class="panel wide">
        <div class="panel-head">
          <div>
            <h2>Projects</h2>
            <p class="hint">Logged in as {$authStore.user?.email}</p>
          </div>
          <button class="ghost" on:click={logout}>Logout</button>
        </div>

        <div class="project-form">
          <input
            placeholder="Project name"
            bind:value={projectName}
            maxlength="80"
          />
          <input
            placeholder="Project description"
            bind:value={projectDescription}
            maxlength="500"
          />
          <button
            on:click={createProject}
            disabled={!projectName.trim() || $projectStore.isLoading}
          >
            Create project
          </button>
        </div>

        {#if $projectStore.error}
          <p class="error">{$projectStore.error}</p>
        {/if}

        <ul class="project-list">
          {#if $projectStore.items.length === 0}
            <li class="empty">No projects yet. Create one to start editing.</li>
          {/if}

          {#each $projectStore.items as project}
            <li>
              <div>
                <h3>{project.name}</h3>
                <p>{project.description ?? 'No description'}</p>
              </div>
              <button
                class="danger"
                on:click={() => deleteProject(project.id)}
                disabled={$projectStore.isLoading}
              >
                Delete
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    font-family: 'Space Grotesk', 'Avenir Next', 'Segoe UI', sans-serif;
    color: #121826;
    background: radial-gradient(circle at 20% 20%, #eaf8ff 0%, #dce9ff 28%, #eef5ff 63%, #f8fcff 100%);
  }

  .page {
    min-height: 100vh;
    position: relative;
    overflow: hidden;
    padding: 2rem 1rem 3rem;
  }

  .nebula {
    position: absolute;
    inset: -20% auto auto -10%;
    width: 28rem;
    height: 28rem;
    border-radius: 999px;
    background: radial-gradient(circle, rgba(58, 214, 255, 0.25) 0%, rgba(58, 214, 255, 0) 70%);
    pointer-events: none;
  }

  .shell {
    width: min(880px, 100%);
    margin: 0 auto;
    display: grid;
    gap: 1.25rem;
    position: relative;
    z-index: 1;
  }

  .hero {
    background: rgba(255, 255, 255, 0.82);
    border: 1px solid #d8e5ff;
    border-radius: 1.1rem;
    padding: 1.5rem;
    backdrop-filter: blur(14px);
  }

  .hero h1 {
    margin: 0.35rem 0 0.7rem;
    font-size: clamp(1.5rem, 4vw, 2.2rem);
    line-height: 1.15;
  }

  .hero p {
    margin: 0;
    color: #445069;
  }

  .badge {
    display: inline-block;
    margin: 0;
    padding: 0.2rem 0.65rem;
    border-radius: 999px;
    background: #d5edff;
    color: #0a4a76;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .panel {
    background: rgba(255, 255, 255, 0.88);
    border: 1px solid #d8e5ff;
    border-radius: 1.1rem;
    padding: 1.3rem;
    display: grid;
    gap: 0.65rem;
    box-shadow: 0 24px 60px -40px rgba(10, 74, 118, 0.35);
  }

  .panel.wide {
    gap: 1rem;
  }

  .panel-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .panel h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .hint {
    margin: 0.1rem 0 0;
    color: #4d5a74;
    font-size: 0.9rem;
  }

  label {
    font-size: 0.83rem;
    color: #2b3650;
    font-weight: 600;
  }

  input {
    border: 1px solid #c8d7f5;
    border-radius: 0.72rem;
    padding: 0.7rem 0.8rem;
    font-size: 0.95rem;
    background: #f7faff;
    color: #1a2132;
  }

  input:focus {
    outline: 2px solid #3ad6ff;
    border-color: #1f97d8;
  }

  button {
    border: 0;
    border-radius: 0.75rem;
    background: linear-gradient(120deg, #1f97d8 0%, #0072bd 100%);
    color: #fff;
    font-weight: 700;
    padding: 0.72rem 0.95rem;
    cursor: pointer;
    transition: transform 120ms ease, opacity 120ms ease;
  }

  button:hover {
    transform: translateY(-1px);
  }

  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
  }

  .ghost {
    background: #e8f3ff;
    color: #0a4a76;
  }

  .danger {
    background: linear-gradient(120deg, #d94848 0%, #bb2a2a 100%);
  }

  .project-form {
    display: grid;
    gap: 0.65rem;
    grid-template-columns: 1fr;
  }

  .project-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.65rem;
  }

  .project-list li {
    border: 1px solid #d8e5ff;
    border-radius: 0.9rem;
    background: #fbfdff;
    padding: 0.8rem;
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
  }

  .project-list h3 {
    margin: 0;
    font-size: 1rem;
  }

  .project-list p {
    margin: 0.25rem 0 0;
    color: #4d5a74;
    font-size: 0.9rem;
  }

  .empty {
    color: #4d5a74;
    font-style: italic;
    justify-content: flex-start;
  }

  .error {
    margin: 0;
    color: #aa2a2a;
    font-size: 0.88rem;
    font-weight: 600;
  }

  @media (min-width: 800px) {
    .project-form {
      grid-template-columns: 1.1fr 1.2fr auto;
      align-items: center;
    }
  }
</style>
