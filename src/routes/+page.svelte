<script>
	import Register from 'flowbite-svelte-blocks/Register.svelte';
	import Section from 'flowbite-svelte-blocks/Section.svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import { Button, Label, Input } from 'flowbite-svelte';
</script>

<Section name="login">
	<Register href="/">
		<svelte:fragment slot="top">MimiDo</svelte:fragment>
		<Tabs>
			<TabItem open title="Login">
				<div class="space-y-4 p-6 sm:p-8 md:space-y-6"></div>
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
