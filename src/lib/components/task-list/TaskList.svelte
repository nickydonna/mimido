<script lang="ts">
  import { invalidateAll } from "$app/navigation";
  import { commands, type VTodo } from "../../../bindings";
  import GlassCheckbox from "../glass-checkbox/GlassCheckbox.svelte";

  let { tasks }: { tasks: VTodo[] } = $props();
  let loading = $state({} as Record<number, boolean>);

  async function toggleDone(task: VTodo) {
    loading[task.id] = true;
    await commands.setVtodoStatus(
      task.id,
      task.status === "Done" ? "InProgress" : "Done",
    );
    loading[task.id] = false;
    await invalidateAll();
  }
</script>

<div>
  {#each tasks as task}
    <div class="flex p-1">
      <div class="flex-1">
        <GlassCheckbox
          loading={loading[task.id]}
          label={task.summary}
          checked={task.status === "Done"}
          onChange={() => toggleDone(task)}
        ></GlassCheckbox>
      </div>
    </div>
  {/each}
</div>
