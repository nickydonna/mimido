<script lang="ts">
  import { type SizeType } from "flowbite-svelte";
  import {
    type HTMLButtonAttributes,
    type HTMLAnchorAttributes,
  } from "svelte/elements";
  type Sizes = "xxs" | "xs" | "sm" | "md" | "lg" | "xl";

  type BaseProps = {
    size?: Sizes;
    disabled?: boolean;
    loading?: boolean;
  };

  type Props =
    | (HTMLAnchorAttributes & BaseProps)
    | (HTMLButtonAttributes & BaseProps);

  const sizes: Record<Sizes, string> = {
    xxs: "px-2 py-1 text-xs",
    xs: "px-3 py-2 text-xs",
    sm: "px-4 py-2 text-sm",
    md: "px-5 py-2.5 text-sm",
    lg: "px-5 py-3 text-base",
    xl: "px-6 py-3.5 text-base",
  };

  let props: Props = $props();

  let {
    class: className,
    size = "md",
    disabled: propsDisabled,
    loading,
  } = $derived(props);
  let disabled = $derived(propsDisabled || loading);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  let classes = $derived([
    "glass-button",
    disabled ? "text-primary-400" : "text-primary-100",
    "glass-clickable rounded-3xl cursor-pointer",
    sizes[size],
    className,
    {
      loading,
    },
  ]);
</script>

{#if isButtonProps(props)}
  <button {...props} type={props.type ?? "button"} class={classes} {disabled}>
    {@render props.children?.()}
  </button>
{:else}
  <a {...props} class={classes}>
    {@render props.children?.()}
  </a>
{/if}
