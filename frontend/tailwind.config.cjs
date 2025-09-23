/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
		'./index.html',
		'./src/**/*.{js,ts,jsx,tsx}',
		'./node_modules/layerchart/**/*.{svelte,js}'
	],
	theme: {
		extend: {}
	},
	plugins: []
};
