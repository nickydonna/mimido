import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import adapter from '@jill64/sveltekit-adapter-aws'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			name: 'mimido',
			deploy: true,
			architecture: 'lambda-s3',
			domain: {
				fqdn: 'mimido.pirus.io',
				certificateArn: 'arn:aws:acm:us-east-1:623155984954:certificate/f7f8860a-4fa3-4e54-8e38-3ca38efa42d5',
			},
		})
	},

	preprocess: [vitePreprocess({})]
};

export default config;
