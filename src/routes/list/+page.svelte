<script>
	import { enhance } from '$app/forms';
  import { Button, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';

  /** @type {import('./$types').PageData} */
  export let data;

  let loading = false;

  /** @type {import('./$types').SubmitFunction} */
  const onDelete = () => {
    loading = true;
    return async ({ update }) => {
      loading = false;
      update();
    }
  }
</script>


<Table>
  <TableHead>
    <TableHeadCell>Task</TableHeadCell>
    <TableHeadCell>Type</TableHeadCell>
    <TableHeadCell>Actions</TableHeadCell>
  </TableHead>
  <TableBody>
    {#each data.events as event}
    <TableBodyRow>
      <TableBodyCell>{event.title}</TableBodyCell>
      <TableBodyCell>{event.type}</TableBodyCell>
      <TableBodyCell>
        <Button disabled={loading} size="sm" color="alternative"  href="/form/{event.eventId}" class="font-medium text-primary-600 hover:underline dark:text-primary-500">Edit</Button>
        <form class="inline-block" method="POST" action="?/delete" use:enhance={onDelete}>
          <input type="text" name="eventId" value={event.eventId} class="hidden">
          <Button disabled={loading} size="sm" color="red" type="submit">Delete</Button>
        </form>
      </TableBodyCell>
    </TableBodyRow>
    {/each}
  </TableBody>
</Table>