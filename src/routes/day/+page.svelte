<script lang="ts">
  import {
    formatISO,
    getMinutes,
    isSameMinute,
    roundToNearestMinutes,
    startOfDay,
  } from "date-fns";
  import {
    addMinutes,
    eachHourOfInterval,
    format,
    isWithinInterval,
    setHours,
    subDays,
    addDays,
    subSeconds,
  } from "date-fns/fp";
  import { type ParsedEvent } from "../../lib/util";
  import Button from "flowbite-svelte/Button.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";
  import { AngleLeftOutline, AngleRightOutline } from "flowbite-svelte-icons";
  import { type VEvent, type EventType } from "../../bindings";
  import { timeStore } from "../../stores/times";
  import EventCard from "$lib/components/event-card";
  import type { PageProps } from "./$types";
  import GlassButtonGroup from "$lib/components/glass-button-group/GlassButtonGroup.svelte";
  import GlassGrouppedButton from "$lib/components/glass-button-group/GlassGrouppedButton.svelte";

  let { data }: PageProps = $props();
  let { date, events } = $derived(data);

  let dragging = $state<VEvent | undefined>(undefined);
  let currentTimeInView = $state(false);
  let currentTime: Date = $state(new Date());
  let hoverTime: Date | undefined = $state(undefined);
  let tags: string[] = $state([]);
  let tagFilter: string | undefined = $state();
  let current = $derived(startOfDay(date));
  let timeBlocks: Array<{ time: Date; check: (d: Date) => boolean }> =
    $derived.by(() => {
      let start = setHours(8, current);
      let end = setHours(23, current);
      let eachHour = eachHourOfInterval({ start, end })
        .map((d) => [
          d,
          addMinutes(15, d),
          addMinutes(30, d),
          addMinutes(45, d),
        ])
        .flat();
      return eachHour.map((h) => ({
        time: h,
        check: isWithinInterval({
          start: subSeconds(1, h),
          end: subSeconds(1, addMinutes(30, h)),
        }),
      }));
    });

  let timeCheck = (event: ParsedEvent, slotCheck: (d: Date) => boolean) =>
    event.starts_at != null && slotCheck(event.starts_at);

  let sortedEvents: Array<[EventType, Array<ParsedEvent>]> = $derived.by(() => {
    if (events == null) return [];
    return [
      ["Block", events.filter((e) => e.event_type === "Block")],
      ["Event", events.filter((e) => e.event_type === "Event")],
      ["Task", events.filter((e) => e.event_type === "Task")],
      ["Reminder", events.filter((e) => e.event_type === "Reminder")],
    ];
  });

  const modalZIndex = 40;
  const scrollCurrentIntoView = () => {
    document.getElementById("current-time")?.scrollIntoView({
      block: "center",
      behavior: "smooth",
    });
  };

  let timeIndicator: { nearestSlot: Date; offset: number } = $state({
    nearestSlot: new Date(),
    offset: 0,
  });
  timeStore.subscribe((storeTime) => {
    currentTime = storeTime;
    const nearestSlot = roundToNearestMinutes(storeTime, {
      nearestTo: 15,
      roundingMethod: "floor",
    });
    const minutes = getMinutes(storeTime) - getMinutes(nearestSlot);
    timeIndicator = {
      nearestSlot: nearestSlot,
      offset: (minutes * 100) / 15,
    };
  });

  /**
   * Give a time slot find the start and end time grid slot
   * if endTime is null, add 15 min to the start time
   * if the time is not in a 15 min slot, move it to the nearest before
   * if when moving start and end are the same, move the end 15 min later
   */
  function getScheduleSlot(e: ParsedEvent) {
    let startDate = roundToNearestMinutes(e.starts_at, {
      nearestTo: 15,
      roundingMethod: "floor",
    });
    let endTime = e.ends_at ?? addMinutes(15, e.starts_at);
    endTime = roundToNearestMinutes(endTime, {
      nearestTo: 15,
      roundingMethod: "floor",
    });
    if (isSameMinute(startDate, endTime)) {
      endTime = addMinutes(15, startDate);
    }
    return `time-${format("HHmm", startDate)} / time-${format("HHmm", endTime)}`;
  }

  function handleTimeDoubleClick(time: Date) {
    // upsert.create(time);
  }
</script>

<div>
  <div
    class="flex sticky top-0 bg-primary-950 py-3 px-1"
    style:z-index={modalZIndex - 2}
  >
    <div class="flex-1">
      <p class="text-lg md:text-4xl text-primary-200">
        {format("E do MMM yy ", date)}
      </p>
    </div>
    <GlassButtonGroup size="md">
      <GlassGrouppedButton
        href="/day?date={formatISO(subDays(1, startOfDay(date)))}"
      >
        <AngleLeftOutline />
      </GlassGrouppedButton>
      <GlassGrouppedButton href="/day?date={formatISO(startOfDay(currentTime))}"
        >Now</GlassGrouppedButton
      >
      <GlassGrouppedButton
        href="/day?date={formatISO(addDays(1, startOfDay(date)))}"
      >
        <AngleRightOutline />
      </GlassGrouppedButton>
    </GlassButtonGroup>
  </div>
  <div
    class="flex sticky top-0 bg-primary-950 py-3 px-1"
    style:z-index={modalZIndex - 2}
  >
    <div class="schedule flex-1">
      <span
        class="block p-1 pt-2 text-center antialiased bg-primary-950 text-primary-300"
        aria-hidden="true"
        style="grid-column: event; grid-row: tracks;">Events</span
      >
      <span
        class="block p-1 pt-2 text-center antialiased bg-primary-950 text-primary-300"
        aria-hidden="true"
        style="grid-column: task; grid-row: tracks;">Tasks</span
      >
      <span
        class="block p-1 pt-2 text-center antialiased bg-primary-950 text-primary-300"
        aria-hidden="true"
        style="grid-column: reminder; grid-row: tracks;">Reminder</span
      >

      <!-- {#if !currentTimeInView && !dragging} -->
      <!--   <Button -->
      <!--     class="fixed bottom-[2rem] end-6 z-40" -->
      <!--     onclick={scrollCurrentIntoView} -->
      <!--   > -->
      <!--     Current Time -->
      <!--   </Button> -->
      <!-- {/if} -->
      <!-- Time indicator -->
      <div
        class="pointer-events-none"
        style:z-index={modalZIndex - 3}
        style:grid-column="times / reminder"
        style:grid-row="time-{format('HHmm', timeIndicator.nearestSlot)}"
      >
        <div
          class="relative w-full"
          style:top="calc({timeIndicator.offset}% - 25px)"
        >
          <span class="relative px-2 text-pink-600 font-bold">
            {format("HH:mm", currentTime)}
          </span>
        </div>
      </div>
      <!-- Dotted line for current time -->
      <div
        id="current-time"
        class="pointer-events-none"
        style:z-index={modalZIndex - 3}
        style:grid-column="times / reminder"
        style:grid-row="time-{format('HHmm', timeIndicator.nearestSlot)}"
      >
        <div
          style:top="{timeIndicator.offset}%"
          class="relative w-full border-b-2 border border-pink-600"
        ></div>
      </div>

      {#each timeBlocks as { time, check } (time)}
        <h2
          ondblclick={() => !dragging && handleTimeDoubleClick(time)}
          class="time-slot m-0.5 text-center text-xs cursor-pointer select-none"
          class:brightness-50={timeIndicator.nearestSlot >= time}
          style:grid-row={`time-${format("HHmm", time)}`}
        >
          {format("HH:mm", time)}
        </h2>
        <div
          class:hidden={hoverTime !== time}
          class="z-40 text-center rounded-lg font-bold text-lg bg-blue-800"
          style:grid-column="event / reminder"
          style:grid-row="time-{format('HHmm', time)}"
        >
          {format("HH:mm", time)}
        </div>
        <div
          aria-hidden="true"
          class="border-t border-dotted"
          class:z-50={dragging}
          class:border-gray-600={getMinutes(time) === 0}
          class:border-gray-300={getMinutes(time) === 30}
          class:border-gray-800={getMinutes(time) !== 0 &&
            getMinutes(time) !== 30}
          style:grid-column="event /reminder"
          style:grid-row="time-{format('HHmm', time)}"
          ondragenter={() => {
            hoverTime = time;
          }}
          ondrop={(e) => {
            // handleDropOnTime(e, time);
          }}
          ondragover={() => false}
        ></div>
        {#each sortedEvents as [type, events], i}
          {@const isBlockType = type === "Block"}
          {#each events.filter((e) => timeCheck(e, check)) as e, k}
            <div
              tabindex={i * 10 + (k + 1)}
              role="button"
              class="group event-card event-card-{type.toLowerCase()}"
              class:brightness-50={timeIndicator.nearestSlot > time}
              style:grid-row={getScheduleSlot(e)}
              style:z-index={isBlockType ? 0 : k + 1}
            >
              <div class="absolute right-2 hidden group-hover:block"></div>
              {#if e.event_type === "Block"}
                <div class="flex h-full flex-col items-center justify-center">
                  <p
                    class="inline-block text-2xl font-medium text-blue-900 opacity-65"
                  >
                    {e.summary.toUpperCase()}
                  </p>
                </div>
              {:else}
                <EventCard event={e} />
              {/if}
            </div>
          {/each}
        {/each}
      {/each}
    </div>
  </div>
</div>

<style lang="postcss">
  @reference "../../app.css";

  @layer components {
    .event-card {
      @apply p-0.5 relative rounded-lg border shadow-2xl;
    }

    .event-card-block {
      @apply m-px bg-polka-indigo-800 border-indigo-900;
      grid-column: event / reminder;
      z-index: 0;
    }

    .event-card-event {
      @apply glass bg-emerald-600 border-green-900 m-0.5;
      grid-column: event;
    }

    .event-card-task {
      @apply glass m-0.5 bg-pink-600 border-pink-900;
      grid-column: task;
    }
    .event-card-reminder {
      @apply glass m-0.5 bg-blue-600 border-blue-900;
      grid-column: reminder;
    }
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
    margin-right: 0.5em;
    border-right: 1px solid gray;
  }

  .schedule {
    margin: 20px 0;
    display: grid;
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
