<script>
	import '../app.pcss';

	import {
		RectangleListOutline,
		CalendarEditOutline,
		PlusOutline, UserOutline
	} from 'flowbite-svelte-icons';
	import BottomNav from 'flowbite-svelte/BottomNav.svelte';
	import BottomNavItem from 'flowbite-svelte/BottomNavItem.svelte';
	import Navbar from 'flowbite-svelte/Navbar.svelte';
	import NavBrand from 'flowbite-svelte/NavBrand.svelte';
	import NavLi from 'flowbite-svelte/NavLi.svelte';
	import NavUl from 'flowbite-svelte/NavUl.svelte';
	import NavHamburger from 'flowbite-svelte/NavHamburger.svelte';
	import { formatISO } from 'date-fns/fp';
	import { page } from '$app/stores';
	import { pwaAssetsHead } from 'virtual:pwa-assets/head';

	// @ts-expect-error virtual import
	import { pwaInfo } from 'virtual:pwa-info';
	import { onMount } from 'svelte';

	onMount(async () => {
		if (pwaInfo) {
			const { registerSW } = await import('virtual:pwa-register');
			registerSW({
				immediate: true,
				onRegistered(r) {
					// uncomment following code if you want check for updates
					// r && setInterval(() => {
					//    console.log('Checking for sw update')
					//    r.update()
					// }, 20000 /* 20s for testing purposes */)
					console.log(`SW Registered: ${r}`);
				},
				onRegisterError(error) {
					console.log('SW registration error', error);
				}
			});
		}
	});

	$: webManifest = pwaInfo ? pwaInfo.webManifest.linkTag : '';

	/** @type {import('./$types').LayoutData} */
	export let data;

	/** @type {string} */
	let date;
	$: date = $page.url.searchParams.get('date') ?? formatISO(new Date());
</script>

<svelte:head>
	{#if pwaAssetsHead.themeColor}
		<meta name="theme-color" content={pwaAssetsHead.themeColor.content} />
	{/if}
	{#each pwaAssetsHead.links as link}
		<link {...link} />
	{/each}

	{@html webManifest}
</svelte:head>

<div class="container mx-auto h-full">
	<div class="mt-6 mb-16">
		<slot />
	</div>
	{#if data.session.user}
		<BottomNav position="fixed" classInner="grid-cols-4">
			<BottomNavItem btnName="Tasks" href="/list?date={date}">
				<RectangleListOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
			<BottomNavItem btnName="Day" href="/day?date={date}">
				<CalendarEditOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
			<BottomNavItem btnName="Add" href="/form?date={date}">
				<PlusOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
			<BottomNavItem btnName="Account" href="/account">
				<UserOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		</BottomNav>
	{/if}
</div>

{#await import('$lib/components/reload-prompt') then { default: ReloadPrompt }}
	<ReloadPrompt />
{/await}
