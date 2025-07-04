<script lang="ts">
  import type { SizeType } from "flowbite-svelte";
  import { setContext } from "svelte";
  import {
    type HTMLButtonAttributes,
    type HTMLAnchorAttributes,
  } from "svelte/elements";

  type BaseProps = {
    size?: SizeType;
    disabled?: boolean;
  };

  type Props =
    | (HTMLAnchorAttributes & BaseProps)
    | (HTMLButtonAttributes & BaseProps);

  const sizes: Record<SizeType, string> = {
    xs: "p-1/4  gap-1",
    sm: "p-1  gap-2",
    md: "p-1.5  gap-3",
    lg: "p-2  gap-2",
    xl: "p-4  gap-5",
  };

  let props: Props = $props();

  let { class: className, size = "md", disabled = false } = $derived(props);
  setContext<() => { size: SizeType; disabled: boolean }>("group", () => ({
    size,
    disabled,
  }));
  // setContext<Boolean>("disabled", () => disabled);
</script>

<div
  class={[
    "glass-button-group",
    "inline-flex rounded-4xl glassy-shadow",
    `size-${size}`,
    sizes[size],
    className,
  ]}
  role="group"
>
  {@render props.children?.()}
</div>

<style lang="postcss">
  @reference "../../../app";
</style>
