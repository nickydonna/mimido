<script lang="ts">
  import { formatISO, isThisYear, startOfDay } from "date-fns";
  import { format, subDays, addDays } from "date-fns/fp";
  import { formatRelativeDay } from "../../lib/util";
  import {
    AngleLeftOutline,
    AngleRightOutline,
    ListOutline,
    CloseOutline,
  } from "flowbite-svelte-icons";
  import { timeState } from "../../stores/times.svelte";
  import type { PageProps } from "./$types";
  import GlassButtonGroup from "$lib/components/glass-button-group/GlassButtonGroup.svelte";
  import GlassGrouppedButton from "$lib/components/glass-button-group/GlassGrouppedButton.svelte";
  import TaskList from "$lib/components/task-list/TaskList.svelte";
  import GlassIcon from "$lib/components/glass-icon/GlassIcon.svelte";
  import Calendar from "./Calendar.svelte";

  let { data }: PageProps = $props();
  let { date, events, todos, unscheduledTodos } = $derived(data);

  const modalZIndex = 40;

  let relativeDay = $derived(formatRelativeDay(date));
  let formattedDate = $derived.by(() => {
    return isThisYear(date)
      ? format("E do MMM", date)
      : format("E do MMM yy", date);
  });

  let taskDrawerOpen = $state(true);
</script>

<div class="flex">
  <div class="flex-3">
    <div class="day-header" style:z-index={modalZIndex - 2}>
      <div>
        <p
          class="py-1.5 px-5 rounded-3xl text-lg md:text-2xl text-white glassy-shadow"
        >
          {#if relativeDay != null}
            {relativeDay}
            <span class="text-white/30 text-base">
              {formattedDate}
            </span>
          {:else}
            {formattedDate}
          {/if}
        </p>
      </div>
      <div class="flex-1"></div>
      <GlassButtonGroup size="md" class="text-white">
        <GlassGrouppedButton
          href="/day?date={formatISO(subDays(1, startOfDay(date)))}"
        >
          <AngleLeftOutline />
        </GlassGrouppedButton>
        <GlassGrouppedButton
          href="/day?date={formatISO(startOfDay(timeState.time))}"
          >Now</GlassGrouppedButton
        >
        <GlassGrouppedButton
          href="/day?date={formatISO(addDays(1, startOfDay(date)))}"
        >
          <AngleRightOutline />
        </GlassGrouppedButton>
      </GlassButtonGroup>
      <GlassIcon
        class="px-3"
        size="sm"
        onclick={() => {
          taskDrawerOpen = !taskDrawerOpen;
        }}
      >
        {#if taskDrawerOpen}
          <CloseOutline />
        {:else}
          <ListOutline />
        {/if}
      </GlassIcon>
    </div>
    <div
      class="flex relative top-3 bg-primary-950 pl-3 pr-6 px-1"
      style:z-index={modalZIndex - 3}
    >
      <Calendar {date} {todos} {events} />
    </div>
  </div>
  {#if taskDrawerOpen}
    <div class={`flex-1 pt-32 px-4 relative border-l border-l-primary-900`}>
      <div class="sticky top-24">
        <TaskList tasks={unscheduledTodos ?? []} />
      </div>
    </div>
  {/if}
</div>

<style lang="postcss">
  @reference "../../app.css";

  .day-header {
    @apply flex sticky top-0 py-6 px-4 gap-2;
  }

  .event-block {
    @apply p-0 m-px glassy-shadow-primary-800 rounded-xl justify-center pointer-events-none;
    backdrop-filter: blur(0.5px);
    filter: none;
    grid-column: event / reminder;
    z-index: 0;
  }

  .blurred-time {
    /* background-color: rgba(0, 0, 0, 0.4); */
    /* background: rgb(0, 0, 0, 0.4); */
    /* background: linear-gradient(0deg, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0.1) 100%); */
    background: repeating-linear-gradient(
      45deg,
      rgba(0, 0, 0, 0.2),
      rgba(0, 0, 0, 0.2) 10px,
      rgba(0, 0, 0, 0.3) 10px,
      rgba(0, 0, 0, 0.3) 20px
    );
  }

  /* .card__bg-work { */
  /*   background-position: center; */
  /*   background-image: url("$lib/assets/work.jpg"); */
  /* } */

  /** Taken from https://css-tricks.com/building-a-conference-schedule-with-css-grid/ */

  .time-slot {
    grid-column: times;
    @apply text-center text-xs cursor-pointer select-none;
    @apply px-3 py-1 mr-0.5 border-r border-slate-100/50;
  }

  .schedule {
    @apply my-3 grid;
    grid-template-columns:
      [times] 4em
      [event-start] 1fr
      [event-end task-start] 1fr
      [task-end reminder-start] 1fr
      [reminder-end];
    grid-template-rows:
      [tracks] auto
      [time-0800] 1fr
      [time-0815] 1fr
      [time-0830] 1fr
      [time-0845] 1fr
      [time-0900] 1fr
      [time-0915] 1fr
      [time-0930] 1fr
      [time-0945] 1fr
      [time-1000] 1fr
      [time-1015] 1fr
      [time-1030] 1fr
      [time-1045] 1fr
      [time-1100] 1fr
      [time-1115] 1fr
      [time-1130] 1fr
      [time-1145] 1fr
      [time-1200] 1fr
      [time-1215] 1fr
      [time-1230] 1fr
      [time-1245] 1fr
      [time-1300] 1fr
      [time-1315] 1fr
      [time-1330] 1fr
      [time-1345] 1fr
      [time-1400] 1fr
      [time-1415] 1fr
      [time-1430] 1fr
      [time-1445] 1fr
      [time-1500] 1fr
      [time-1515] 1fr
      [time-1530] 1fr
      [time-1545] 1fr
      [time-1600] 1fr
      [time-1615] 1fr
      [time-1630] 1fr
      [time-1645] 1fr
      [time-1700] 1fr
      [time-1715] 1fr
      [time-1730] 1fr
      [time-1745] 1fr
      [time-1800] 1fr
      [time-1815] 1fr
      [time-1830] 1fr
      [time-1845] 1fr
      [time-1900] 1fr
      [time-1915] 1fr
      [time-1930] 1fr
      [time-1945] 1fr
      [time-2000] 1fr
      [time-2015] 1fr
      [time-2030] 1fr
      [time-2045] 1fr
      [time-2100] 1fr
      [time-2115] 1fr
      [time-2130] 1fr
      [time-2145] 1fr
      [time-2200] 1fr
      [time-2215] 1fr
      [time-2230] 1fr
      [time-2245] 1fr
      [time-2300] 1fr
      [time-2315] 1fr
      [time-2330] 1fr
      [time-2345] 1fr
      [time-0000] 1fr;
  }
</style>
