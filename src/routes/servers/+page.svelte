<script lang="ts">
  import {
    Card,
    Button,
    Input,
    Label,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    Spinner,
  } from "flowbite-svelte";
  import { commands, type Server, type Result } from "../../bindings";
  import { invalidateAll } from "$app/navigation";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();

  let { servers } = $derived(data);

  let syncingCalendars = $state(false);
  const syncAllCalendars = async () => {
    syncingCalendars = true;
    await commands.syncAllCalendars();
    await invalidateAll();
    syncingCalendars = false;
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
</script>

<Card padding="sm" class="mx-auto">
  <div class="flex flex-col items-center pb-4">
    <!-- <Avatar size="lg" src={frog} /> -->
    <h5 class="mb-1 text-xl font-medium text-gray-900 dark:text-white">Mimi</h5>
    <span class="text-sm text-gray-500 dark:text-gray-400">A Mimi</span>
    <div class="mt-4 flex space-x-3 lg:mt-6 rtl:space-x-reverse">
      <Button
        onclick={syncAllCalendars}
        color="light"
        class="dark:text-white"
        disabled={syncingCalendars}
      >
        {#if syncingCalendars}
          <Spinner class="me-3" size="4" />
        {/if}
        Resync
      </Button>
    </div>
  </div>
</Card>
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
  <Table class="mt-3">
    <caption
      class="border-b border-gray-400 bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white"
    >
      Added Servers
    </caption>
    <TableBody>
      {#each servers as server}
        <TableBodyRow>
          <TableBodyCell>
            {server.server_url}
          </TableBodyCell>
          <TableBodyCell>
            {server.user}
          </TableBodyCell>
          <TableBodyCell>
            {#if server.last_sync == null}
              Never Synced
            {:else}
              Synced on: {new Date(server.last_sync).toLocaleString("en-UK", {
                dateStyle: "short",
                timeStyle: "short",
              })}
            {/if}
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
{/if}
<form class="flex flex-col space-y-6 mt-5" onsubmit={addServer}>
  <h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">
    Add Server
  </h3>
  {#if creating}
    <div class="flex justify-center">
      <div>
        <Spinner size="md" />
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
        type="url"
        name="serverUrl"
        placeholder="https://caldav.fastmail.com/"
        required
      />
    </Label>
    <Label class="space-y-2">
      <span>Your email</span>
      <Input type="email" name="user" placeholder="name@company.com" required />
    </Label>
    <Label class="space-y-2">
      <span>Your password</span>
      <Input type="password" name="password" placeholder="•••••" required />
    </Label>
    <Button type="submit" class="w-full1">Connect</Button>
  {/if}
</form>
