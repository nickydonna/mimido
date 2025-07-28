<script lang="ts">
  import { invalidateAll } from "$app/navigation";
  import { commands, type VTodo } from "../../../bindings";
  import GlassButton from "../glass-button/GlassButton.svelte";

  let { tasks }: { tasks: VTodo[] } = $props();
  let loading = $state(false);
  async function toggleDone(task: VTodo) {
    loading = true;
    await commands.setVtodoStatus(
      task.id,
      task.status === "Done" ? "InProgress" : "Done",
    );
    loading = false;
    await invalidateAll();
  }
</script>

<div>
  {#each tasks as task}
    <div class="flex p-1">
      <div class="flex-1">
        {task.summary}
      </div>
      <div>
        <GlassButton {loading} size="xs" onclick={() => toggleDone(task)}
          >D</GlassButton
        >
      </div>
    </div>
  {/each}
</div>
