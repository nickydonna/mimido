<script lang="ts">
	/* eslint-disable svelte/no-at-html-tags */
	import '../app.pcss';
	import Keydown from 'svelte-keydown';

	import { navigating } from '$app/stores';
	import { expoOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

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
	import { isLoading, isUpserting, upsert, selectedEvent } from '$lib/stores';

	// @ts-expect-error virtual import
	import { pwaInfo } from 'virtual:pwa-info';
	import type { LayoutData } from './$types';
	import DetailModal from '$lib/components/details-modal';
	// Move to store

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

	export let data: LayoutData;

	let transitionParams = {
		y: 320,
		duration: 200,
		easing: sineIn
	};

	$: webManifest = pwaInfo ? pwaInfo.webManifest.linkTag : '';

	function closeDrawer() {
		upsert.reset();
	}

	let date: string;
	$: date = $page.url.searchParams.get('date') ?? formatISO(new Date());

	function handleKey(e: CustomEvent<string>): void {
		if (e.detail === 'a') {
			setTimeout(() => upsert.create(), 1);
			return;
		}
		if (e.detail === 'Escape' && isUpserting) {
			upsert.reset();
			return;
		}
		if (e.detail === 'Escape' && $selectedEvent) {
			$selectedEvent = undefined;
			return;
		}
		if (e.detail === 'e' && $selectedEvent) {
			upsert.update($selectedEvent);
		}
	}
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
{#if $navigating || $isLoading}
	<!--
		Loading animation for next page since svelte doesn't show any indicator.
		 - delay 100ms because most page loads are instant, and we don't want to flash
		 - long 12s duration because we don't actually know how long it will take
		 - exponential easing so fast loads (>100ms and <1s) still see enough progress,
			 while slow networks see it moving for a full 12 seconds
	-->
	<div
		class="fixed top-0 right-0 left-0 h-2 rounded-r-lg z-[100] bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500"
		in:slide={{ delay: 50, duration: 12000, axis: 'x', easing: expoOut }}
	></div>
{/if}
<div class="container mx-auto h-full">
	<div class="mt-6 mb-16">
		<slot />
	</div>
</div>
{#if data.loggedIn}
	<Keydown pauseOnInput={!$selectedEvent} on:key={handleKey}></Keydown>
	<DetailModal
		event={$selectedEvent}
		on:close={() => ($selectedEvent = undefined)}
		on:delete={() => ($selectedEvent = undefined)}
	/>
	<Drawer
		width="w-full"
		transitionType="fly"
		placement="bottom"
		{transitionParams}
		hidden={!$isUpserting}
	>
		<div class="mb-10">
			<TaskForm on:success={closeDrawer} />
		</div>
	</Drawer>
	<BottomNav position="fixed" classInner="grid-cols-4" navType="application">
		<BottomNavItem btnName="Tasks" href="/list?date={date}" appBtnPosition="left">
			<RectangleListOutline
				class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
			/>
		</BottomNavItem>
		<BottomNavItem btnName="Day" href="/day?date={date}" appBtnPosition="middle">
			<CalendarEditOutline
				class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
			/>
		</BottomNavItem>
		{#if !$isUpserting}
			<BottomNavItem btnName="Add" on:click={() => upsert.create()} appBtnPosition="middle">
				<PlusOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		{:else}
			<BottomNavItem btnName="Close" on:click={closeDrawer} appBtnPosition="middle">
				<CloseOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		{/if}
		<BottomNavItem btnName="Account" href="/account" appBtnPosition="right">
			<UserOutline
				class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
			/>
		</BottomNavItem>
	</BottomNav>
{/if}

{#await import('$lib/components/reload-prompt/index.js') then { default: ReloadPrompt }}
	<ReloadPrompt />
{/await}
