<script lang="ts">
  import { invalidateAll } from "$app/navigation";
  import { formatISO } from "date-fns";
  import { commands, type VTodo } from "../../../bindings";
  import { timeState } from "../../../stores/times.svelte";
  import GlassCheckbox from "../glass-checkbox/GlassCheckbox.svelte";
  import DisclosureGroup from "../disclosure/DisclosureGroup.svelte";
  import Disclosure from "../disclosure/Disclosure.svelte";
  import {
    EventUpsert,
    eventUpserter,
  } from "../../../stores/eventUpserter.svelte";
  import type { UnscheduledTask } from "$lib/util";
  import {
    EventDragger,
    eventDragger,
  } from "../../../stores/eventDragger.svelte";
  const NO_CAT = "[No Category]";

  let { tasks }: { tasks: Array<UnscheduledTask> } = $props();
  let loading = $state({} as Record<number, boolean>);

  let groupped = $derived.by(() => {
    const obj = tasks.reduce(
      (acc, task) => {
        const tags = task.tag ? task.tag.split(",") : [NO_CAT];
        let newAcc = { ...acc };
        tags.forEach((tag) => {
          const oldVal = newAcc[tag] ?? [];
          newAcc = {
            ...newAcc,
            [tag]: [...oldVal, task],
          };
        });
        return newAcc;
      },
      { [NO_CAT]: [] } as Record<string, UnscheduledTask[]>,
    );
    return Object.entries(obj).sort((a, b) => {
      if (a[0] === NO_CAT) return 1;
      if (b[0] === NO_CAT) return -1;
      if (a[0] < b[0]) {
        return -1;
      }
      if (a[0] > b[0]) {
        return 1;
      }
      return 0;
    });
  });

  async function toggleDone(task: VTodo) {
    loading[task.id] = true;
    await commands.setVcmpStatus(
      task.id,
      task.status === "Done" ? "InProgress" : "Done",
      formatISO(timeState.time),
    );
    loading[task.id] = false;
    await invalidateAll();
  }

  function handleDrag(task: UnscheduledTask) {
    eventDragger.state = EventDragger.Dragging(task);
  }

  function handleDragEnd(e: Event, task: UnscheduledTask) {
    // e.preventDefault();
    eventDragger.state = EventDragger.None;
  }
</script>

<DisclosureGroup role="tree" aria-rowcount={tasks.length}>
  {#each groupped as [title, tasks], j}
    <div role="rowgroup">
      <Disclosure label={title}>
        {#snippet header()}
          <div class="p-1.5">
            {#if title !== NO_CAT}#{/if}
            {title}
          </div>
        {/snippet}
        {#snippet content()}
          <div role="row">
            {#each tasks as task, i}
              {@const isDone = task.status === "Done"}
              <div
                tabindex={j * 10 + i}
                role="cell"
                draggable="true"
                ondragstart={() => handleDrag(task)}
                ondragend={(e) => handleDragEnd(e, task)}
                class="flex p-2 m-1 border-b border-primary-500 last:border-none"
              >
                <div class="mt-1">
                  <GlassCheckbox
                    loading={loading[task.id]}
                    label={""}
                    checked={isDone}
                    onChange={() => toggleDone(task)}
                  ></GlassCheckbox>
                </div>

                <button
                  class="flex-1"
                  class:line-through={isDone}
                  onclick={() =>
                    (eventUpserter.state = EventUpsert.Updating(task))}
                >
                  {task.summary}
                </button>
              </div>
            {/each}
          </div>
        {/snippet}
      </Disclosure>
    </div>
  {/each}
</DisclosureGroup>
