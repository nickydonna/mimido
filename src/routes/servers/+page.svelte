<script lang="ts">
  import { Button, Input, Label, Spinner } from "flowbite-svelte";
  import {
    commands,
    type Server,
    type Result,
    type Calendar,
  } from "../../bindings";
  import { invalidateAll } from "$app/navigation";
  import type { PageProps } from "./$types";
  import GlassButton from "$lib/components/glass-button/GlassButton.svelte";
  import DisclosureGroup from "$lib/components/disclosure/DisclosureGroup.svelte";
  import Disclosure from "$lib/components/disclosure/Disclosure.svelte";
  import Toggle from "$lib/components/toggle/Toggle.svelte";

  let { data }: PageProps = $props();

  let { servers } = $derived(data);

  let loadingCalendars = $state(false);
  const syncAllCalendars = async () => {
    loadingCalendars = true;
    await commands.syncAllCalendars();
    await invalidateAll();
    loadingCalendars = false;
  };

  let result = $state<Result<Server, string> | null>(null);
  let creating = $state(false);

  async function addServer(
    event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement },
  ) {
    event.preventDefault();
    creating = true;
    result = null;
    const data = new FormData(event.currentTarget);
    const server = data.get("serverUrl") as string;
    const user = data.get("user") as string;
    const password = data.get("password") as string;
    const response = await commands.createServer(server, user, password);
    result = response;
    await invalidateAll();
    creating = false;
  }

  async function handleDefaultChange(
    calendar: Calendar,
    setAsDefault: boolean,
  ) {
    if (!setAsDefault) {
      return;
    }
    loadingCalendars = true;
    await commands.setDefaultCalendar(calendar.id);
    await invalidateAll();
    loadingCalendars = false;
  }
</script>

<div>
  <div class="flex items-center">
    <div class="flex-1">
      <h1 class="text-lg md:text-4xl text-primary-200">Configuration</h1>
    </div>
    <div>
      <GlassButton onclick={syncAllCalendars} loading={loadingCalendars}>
        Sync
      </GlassButton>
    </div>
  </div>
  <div class="my-5 h-0.5 bg-primary-100/30 -mx-5 rounded"></div>
  {#if servers.length === 0}
    <div class="flex flex-col items-center">
      <h3 class="mb-2 text-xl font-medium text-gray-900 dark:text-white">
        No servers found
      </h3>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        Add a server to start syncing your calendars.
      </p>
    </div>
  {:else}
    <h2 class="text-lg md:text-2xl text-primary-200">Added Servers</h2>
    <DisclosureGroup>
      {#each servers as [server, calendars], index}
        <Disclosure label={`Server for ${server.user}`} expanded={index === 0}>
          {#snippet header()}
            <div class="flex">
              <div
                class="border-b p-4 pl-8 text-primary-300 dark:border-primary-700"
              >
                {server.server_url}
              </div>
              <div
                class="border-b p-4 pl-8 text-primary-300 dark:border-primary-700"
              >
                {server.user}
              </div>
              <div
                class="border-b p-4 pl-8 text-primary-300 dark:border-primary-700"
              >
                {#if server.last_sync == null}
                  Never Synced
                {:else}
                  Synced: {new Date(server.last_sync).toLocaleString("en-UK", {
                    dateStyle: "short",
                    timeStyle: "short",
                  })}
                {/if}
              </div>
            </div>
          {/snippet}
          {#snippet content()}
            {#each calendars as calendar}
              <div
                class="flex border-b p-4 pl-8 text-primary-300 dark:border-primary-700"
              >
                <div class="flex-1">
                  {calendar.name}
                </div>
                <div>
                  <Toggle
                    label="Default Calendar"
                    disabled={loadingCalendars}
                    checked={calendar.default_value}
                    onchange={(value) => handleDefaultChange(calendar, value)}
                  />
                </div>
              </div>
            {/each}
          {/snippet}
        </Disclosure>
      {/each}
    </DisclosureGroup>
  {/if}
  <form class="flex flex-col space-y-6 mt-5" onsubmit={addServer}>
    <h3 class="p-0 text-xl font-medium text-primary-200">Add Server</h3>
    {#if creating}
      <div class="flex justify-center">
        <div>
          <Spinner />
          <h4>Creating server ...</h4>
        </div>
      </div>
    {:else}
      {#if result}
        {#if result.status === "error"}
          <div class="flex justify-center">
            <div>
              <h4 class="text-red-500">{result.error}</h4>
            </div>
          </div>
        {:else}
          <div class="flex justify-center">
            <div>
              <h4 class="text-green-500">Server created successfully</h4>
            </div>
          </div>
        {/if}
      {/if}
      <Label class="space-y-2">
        <span>Only CalDAV with user/password is supported for now</span>
        <Input
          class="!bg-primary-900"
          color="primary"
          type="url"
          name="serverUrl"
          placeholder="https://caldav.fastmail.com/"
          required
        />
      </Label>
      <Label class="space-y-2">
        <span>Your email</span>
        <Input
          class="!bg-primary-900"
          color="primary"
          type="email"
          name="user"
          placeholder="name@company.com"
          required
        />
      </Label>
      <Label class="space-y-2">
        <span>Your password</span>
        <Input
          class="!bg-primary-900"
          color="primary"
          type="password"
          name="password"
          placeholder="•••••"
          required
        />
      </Label>
      <Button type="submit" class="w-full1">Connect</Button>
    {/if}
  </form>
</div>
