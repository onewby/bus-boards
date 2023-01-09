import { sveltekit } from '@sveltejs/kit/vite';
import {SvelteKitPWA} from "@vite-pwa/sveltekit";

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [
        sveltekit(),
        SvelteKitPWA({
            registerType: 'autoUpdate',
            manifest: {
                "name": "Bus Boards",
                "short_name": "Bus Boards",
                "description": "Find and track bus times",
                "theme_color": "#f59e0b",
                "background_color": "#f59e0b",
                "display": "standalone",
                "scope": "/",
                "start_url": "/",
                "id": "/",
                "icons": [
                    { "src": "/favicon.ico", "type": "image/x-icon", "sizes": "32x32" },
                    { "src": "/icon-192.png", "type": "image/png", "sizes": "192x192" },
                    { "src": "/icon-512.png", "type": "image/png", "sizes": "512x512" },
                    { "src": "/icon-192-maskable.png", "type": "image/png", "sizes": "192x192", "purpose": "maskable" },
                    { "src": "/icon-512-maskable.png", "type": "image/png", "sizes": "512x512", "purpose": "maskable" }
                ]
            },
            workbox: {
                globPatterns: ['**/*.{js,css,html,ico,png,svg}']
            },
            devOptions: {
                enabled: true,
                type: 'module',
                navigateFallback: '/',
            },
        })
    ]
};

export default config;
