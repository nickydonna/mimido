const flattenColorPalette = require('tailwindcss/lib/util/flattenColorPalette')
const Color = require('color');

const lighten = (clr, val) => Color(clr).lighten(val).rgb().string()

const includeColors = [
	'indigo', 'blue', 'red', 'yellow', 'cyan', 'green',
	'emerald', 'green', 'teal', 'sky', 'violet', 'rose', 'teal',
	'amber', 'fuchsia'
];

/** @type {import('tailwindcss').Config}*/
const config = {
	content: ['./src/**/*.{html,js,svelte,ts}', './node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'],
	darkMode: 'class',
	safelist: [
		...includeColors.map(c => [
			`bg-${c}-600`,
			`border-${c}-900`,
			`bg-polka-${c}-800`,
			`text-${c}-600`
		]).flat(),
		"z-[11000]",
	],
	theme: {
		extend: {
			colors: {
				// flowbite-svelte
				primary: {
					'50': '#f0fdfa',
					'100': '#ccfbf1',
					'200': '#99f6e4',
					'300': '#5eead4',
					'400': '#2dd4bf',
					'500': '#14b8a6',
					'600': '#0d9488',
					'700': '#0f766e',
					'800': '#115e59',
					'900': '#134e4a'
				}
			}
		}
	},
	plugins: [
		require('flowbite/plugin'),
		require('@tailwindcss/typography'),
		function ({ matchUtilities, theme }) {
      matchUtilities(
        {
           // Class name
					'bg-polka': (value) => {
						return {
							'background-image': `radial-gradient(${value} 1px, transparent 1px)`,
							'background-color': `${lighten(value, 0.5)}`,
							'background-size': "16px 16px",
						}
					}
        },
        // Default values.
        // `flattenColorPalette` required to support native Tailwind color classes like `red-500`, `amber-300`, etc. 
        // In most cases you may just pass `theme('config-key')`, where `config-key` could be any (`spacing`, `fontFamily`, `foo`, `bar`)
				{
					values: flattenColorPalette.default(theme('colors')),
					type: ['color', 'any'],
				} 
      )
    },
	]
};

module.exports = config;
