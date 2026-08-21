import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://ce-dot-net.github.io',
  base: '/ace-sdk/rust/core',
  integrations: [
    starlight({
      title: 'ace-sdk-core (Rust)',
      description: 'Rust client library for ACE pattern learning',
      defaultLocale: 'root',
      locales: {
        root: { label: 'English', lang: 'en' },
      },
      social: {
        github: 'https://github.com/ce-dot-net/ace-sdk'
      },
      sidebar: [
        {
          label: 'Guide',
          items: [
            { label: 'Getting Started', slug: 'guide/getting-started' },
            { label: 'Architecture', slug: 'guide/architecture' },
            { label: 'ACE Client', slug: 'guide/ace-client' },
            { label: 'Authentication', slug: 'guide/authentication' },
            { label: 'Caching', slug: 'guide/caching' },
            { label: 'Configuration', slug: 'guide/configuration' },
            { label: 'Traces (read)', slug: 'guide/traces' }
          ]
        },
        {
          label: 'API Reference',
          autogenerate: { directory: 'api' }
        }
      ],
      editLink: {
        baseUrl: 'https://github.com/ce-dot-net/ace-sdk/edit/main/sdks/rust/core/docs/'
      }
    })
  ]
});
