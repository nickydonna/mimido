<script>
	import Register from 'flowbite-svelte-blocks/Register.svelte';
	import Section from 'flowbite-svelte-blocks/Section.svelte';
	import { Tabs, TabItem, Spinner } from 'flowbite-svelte';
	import { Button, Label, Input } from 'flowbite-svelte';
	import { enhance } from '$app/forms';


	let signingIn = false;
	let newPasswordRequired = false;
	/** @type {string | undefined }*/
	let sessionId = undefined;
	let email = '';
</script>

<Section name="login">
	<Register href="/">
		<svelte:fragment slot="top">MimiDo</svelte:fragment>
		<Tabs>
			<TabItem open title="Login">
				<div class="space-y-4 p-6 sm:p-8 md:space-y-6">
	
					<!-- <form class="flex flex-col space-y-6" method="POST" action="?/login">
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Login</h3>
						<Label class="space-y-2">
							<span>CalDav Server</span>
							<Input type="url" name="server" placeholder="https://caldav.fastmail.com/" required />
						</Label>
						<Label class="space-y-2">
							<span>Your email</span>
							<Input type="email" name="email" placeholder="name@company.com" required />
						</Label>
						<Label class="space-y-2">
							<span>Your password</span>
							<Input type="password" name="password" placeholder="•••••" required />
						</Label>
						<Label class="space-y-2">
							<span>Calendar</span>
							<Input type="text" name="calendar" placeholder="Calendar to use as store" required />
						</Label>
						<Button type="submit" class="w-full1">Connect</Button>
					</form> -->
				</div>
			</TabItem>
			<TabItem title="Import Token">
				<div class="space-y-4 p-6 sm:p-8 md:space-y-6">
					<form class="flex flex-col space-y-6" method="POST" action="?/import">
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">From Token</h3>
						<Label class="space-y-2">
							<span>Token</span>
							<Input type="password" name="token" placeholder="...." required />
						</Label>
						<Button type="submit" class="w-full1">Connect</Button>
					</form>
				</div>
			</TabItem>
		</Tabs>
	</Register>
</Section>
