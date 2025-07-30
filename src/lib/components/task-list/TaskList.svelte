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
    <div class="flex p-2 m-1 border-b border-primary-500 last:border-none">
      <div class="mt-1">
        <GlassCheckbox
          loading={loading[task.id]}
          label={""}
          checked={task.status === "Done"}
          onChange={() => toggleDone(task)}
        ></GlassCheckbox>
      </div>

      <div class="flex-1">
        {task.summary}
      </div>
    </div>
  {/each}
</div>
