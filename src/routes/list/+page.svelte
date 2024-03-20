<script>
  import { Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';

  /** @type {import('./$types').PageData} */
  export let data;
</script>


<Table>
  <TableHead>
    <TableHeadCell>Task</TableHeadCell>
    <TableHeadCell>Type</TableHeadCell>
  </TableHead>
  <TableBody>
    {#each data.events as event}
    <TableBodyRow>
      <TableBodyCell>{event.title}</TableBodyCell>
      <TableBodyCell>{event.type}</TableBodyCell>
    </TableBodyRow>
    {/each}
  </TableBody>
</Table>