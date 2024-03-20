<script>
  import { Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';

  /** @type {import('./$types').PageData} */
  export let data;
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
        <a href="/form/{event.eventId}" class="font-medium text-primary-600 hover:underline dark:text-primary-500">Edit</a>
      </TableBodyCell>
    </TableBodyRow>
    {/each}
  </TableBody>
</Table>