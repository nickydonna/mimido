<script lang="ts">
	import '../app.pcss';

	import {
		RectangleListOutline,
		CalendarEditOutline,
		PlusOutline,
		UserOutline,
		CloseOutline
	} from 'flowbite-svelte-icons';
	import BottomNav from 'flowbite-svelte/BottomNav.svelte';
	import BottomNavItem from 'flowbite-svelte/BottomNavItem.svelte';

	import { sineIn } from 'svelte/easing';
	import { formatISO } from 'date-fns/fp';
	import { page } from '$app/stores';
	import { pwaAssetsHead } from 'virtual:pwa-assets/head';
	import { onMount } from 'svelte';
	import { Drawer } from 'flowbite-svelte';
	import TaskForm from '$lib/components/task-form';
	import { selectedEvent } from '$lib/stores';
	import type { LayoutData } from './$types';

	// @ts-expect-error virtual import
	import { pwaInfo } from 'virtual:pwa-info';

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
	let transitionParams = {
		y: 320,
		duration: 200,
		easing: sineIn
	};
	let hideUpsertDrawer = true;

	$: webManifest = pwaInfo ? pwaInfo.webManifest.linkTag : '';

	export let data: LayoutData;
	$: {
		if ($selectedEvent) {
			hideUpsertDrawer = false;
		}
	}

	function closeDrawer() {
		if ($selectedEvent) {
			selectedEvent.set(undefined);
		}
		hideUpsertDrawer = true;
	}

	let date: string;
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
</div>
{#if data.loggedIn}
	<Drawer
		transitionType="fly"
		placement="bottom"
		{transitionParams}
		width="w-full"
		bottomOffset="bottom-8"
		bind:hidden={hideUpsertDrawer}
	>
		<TaskForm event={$selectedEvent} on:success={closeDrawer} />
	</Drawer>
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
		{#if hideUpsertDrawer}
			<BottomNavItem btnName="Add" on:click={() => (hideUpsertDrawer = false)}>
				<PlusOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		{:else}
			<BottomNavItem btnName="Close" on:click={closeDrawer}>
				<CloseOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		{/if}
		<BottomNavItem btnName="Account" href="/account">
			<UserOutline
				class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
			/>
		</BottomNavItem>
	</BottomNav>
{/if}

{#await import('$lib/components/reload-prompt/index.js') then { default: ReloadPrompt }}
	<ReloadPrompt />
{/await}
