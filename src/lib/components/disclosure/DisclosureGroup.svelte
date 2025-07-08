<script lang="ts">
  const props = $props();
</script>

<div class="w-full mt-4">
  <div class="mx-auto w-full rounded-2xl bg-primary-900/50 p-2">
    {@render props.children?.()}
  </div>
</div>
