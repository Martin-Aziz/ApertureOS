import adapter from '@sveltejs/adapter-auto';
import adapterStatic from '@sveltejs/adapter-static';

const isDesktopBuild = process.env.DESKTOP_BUILD === '1';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: isDesktopBuild
      ? adapterStatic({
          fallback: 'index.html'
        })
      : adapter()
  }
};

export default config;
