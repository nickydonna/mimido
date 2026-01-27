<script lang="ts">
  import type { Snippet } from "svelte";
  import type { AriaAttributes, AriaRole } from "svelte/elements";

  interface Props extends AriaAttributes {
    role?: AriaRole;
    children: Snippet;
  }
  const { children, role, ...aria }: Props = $props();
</script>

<div class="w-full mt-4" {role} {...aria}>
  <div class="mx-auto w-full rounded-2xl bg-primary-900/50 p-2">
    {@render children()}
  </div>
</div>
