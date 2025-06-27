<script lang="ts">
  import type { Component } from "svelte";
  import type { ClassValue } from "svelte/elements";
  import { slide } from "svelte/transition";

  const duration = 300;

  const {
    iconCmp: Cmp,
    text,
    ...rest
  } = $props<{ iconCmp: Component; text: string; class?: ClassValue }>();
  let hoverred = $state(false);

  function onEnter() {
    hoverred = true;
  }
  function onExit() {
    hoverred = false;
  }
</script>

{#if !hoverred}
  <span
    class={["text-slate-300", rest.class]}
    role="tooltip"
    onmouseenter={onEnter}
    transition:slide={{
      duration,
      axis: "x",
    }}
  >
    <Cmp onmouseenter={onEnter} />
  </span>
{:else}
  <span
    role="tooltip"
    class={["text-sm text-slate-300", rest.class]}
    onmouseleave={onExit}
    transition:slide={{
      duration,
      axis: "x",
    }}
  >
    {text}
  </span>
{/if}
