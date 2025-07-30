<script lang="ts">
  type Props = {
    label: string;
    checked: boolean;
    disabled?: boolean;
    loading?: boolean;
    onChange: (checked: boolean) => void;
  };
  let {
    label,
    checked,
    onChange,
    disabled: pDisabled = false,
    loading = false,
  }: Props = $props();
  let uid = $props.id();
  let disabled = $derived(loading || pDisabled);
</script>

<button
  class={["glass-checkbox", { disabled, loading }]}
  {disabled}
  onclick={() => onChange(!checked)}
>
  <input
    type="checkbox"
    {disabled}
    class="sr-only"
    id={`${uid}-glass-checkbox`}
    name={label}
    {checked}
  />
  <label for="glass-checkbox">{label}</label>
</button>

<style lang="postcss">
  @reference "../../../app";
  .glass-checkbox input[type="checkbox"] + label {
    display: block;
    position: relative;
    padding-left: 35px;
    cursor: pointer;
    user-select: none;
  }

  .glass-checkbox:hover input[type="checkbox"] + label:before {
    @apply border border-primary-900;
    box-shadow: 3px 1px 0 var(--color-primary-400);
  }

  .glass-checkbox.loading input[type="checkbox"] + label:before {
    animation: spin 2s linear infinite;
  }

  .glass-checkbox input[type="checkbox"] + label:before {
    @apply border border-primary-900;
    content: "";
    display: block;
    width: 1.4em;
    height: 1.4em;
    border-radius: 1em;
    position: absolute;
    left: 0;
    top: 0;
    transition:
      all 0.2s,
      background 0.2s ease-in-out;
    background: var(--color-white);
    box-shadow: -3px -1px 0 var(--color-primary-400);
  }
  .glass-checkbox input[type="checkbox"]:checked + label:before {
    @apply rounded-full border-2 border-primary-100 bg-primary-800;
    width: 1.3em;
    height: 1.3em;
    box-shadow: 3px 1px 0 var(--color-primary-400);
  }
</style>
