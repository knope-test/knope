---
title: CreateChangeFile
---

Create a [change file](/reference/concepts/change-file) interactively. Creates the `.changeset` directory if missing.

## Example

:::note
The [default workflows] include an `document-change` workflow that will run this step for you.
If you don't already have a `knope.toml` file, you don't need to create one to use this feature.
:::

With a `knope.toml` file that looks like this:

```toml
[packages.first]
versioned_files = ["first/Cargo.toml"]
changelog = "first/CHANGELOG.md"
extra_changelog_sections = [
    { name = "Poems 🎭", types = ["poem"] }
]

[packages.second]
versioned_files = ["second/Cargo.toml"]
changelog = "second/CHANGELOG.md"

[[workflows]]
name = "document-change"

[[workflows.steps]]
type = "CreateChangeFile"
```

You could run `knope document-change` to start a new change file.
First, Knope will prompt you to select the packages that this change affects.
For this example, we check off both packages.

```markdown
- [x] first
- [x] second
```

:::note
If there is only one package, Knope skips this step and a special, `default` package is automatically selected.
This is the case when you don't have a `knope.toml` file.
:::

For each package, Knope will prompt you to select the type of change you are documenting.
The available change types are `major`, `minor`, `patch`, and any of the custom types configured in `extra_changelog_sections.types`.
For the `first` package, this will look like:

```
Enter the type for the `first` package:

major
minor
patch
> poem
```

For the `second` package, we would not have the `poem` option. Next, you will be prompted write a short summary of the change (a few words). The summary will be used both as the name of the file and a header in the changelog generated by `PrepareRelease`.

If the summary is \`\[i carry your heart with me(i carry it in]\`, this step will then generate a file `.changeset/i_carry_your_heart_with_mei_carry_it_in.md` with the following contents:

```markdown
---
first: poem
second: major
---

#### `[i carry your heart with me(i carry it in]`
```

If that brief summary isn't enough, you should then edit this file and add more detail below the generated heading, using all the Markdown features you want!

```markdown
---
first: poem
second: major
---

#### `[i carry your heart with me(i carry it in]`

**E. E. Cummings**

<pre>
i carry your heart with me(i carry it in
my heart)i am never without it(anywhere
i go you go,my dear;and whatever is done
by only me is your doing,my darling)
                                   i fear
no fate(for you are my fate,my sweet)i want
no world(for beautiful you are my world,my true)
and it’s you are whatever a moon has always meant
and whatever a sun will always sing is you

here is the deepest secret nobody knows
(here is the root of the root and the bud of the bud
and the sky of the sky of a tree called life;which grows
higher than soul can hope or mind can hide)
and this is the wonder that's keeping the stars apart

i carry your heart(i carry it in my heart)
</pre>
```

When you are done, you can run `knope document-change` again to create another change file.
When you are ready to release, run [`PrepareRelease`] to combine all the change files and conventional commits into a changelog and update the versions of any configured [packages]. The type of the change for each package will decide where it's placed in the changelog: so `first/CHANGELOG.md` will have a `### Poems 🎭` section and `second/CHANGELOG.md` will have a `### Breaking Changes` section, each containing the summary and body of the change.

For completeness, this is what the changelog for `first` would look like (if there had been no other changes):

```markdown
### Poems 🎭

#### `[i carry your heart with me(i carry it in]`

**E. E. Cummings**

<pre>
i carry your heart with me(i carry it in
my heart)i am never without it(anywhere
i go you go,my dear;and whatever is done
by only me is your doing,my darling)
                                   i fear
no fate(for you are my fate,my sweet)i want
no world(for beautiful you are my world,my true)
and it’s you are whatever a moon has always meant
and whatever a sun will always sing is you

here is the deepest secret nobody knows
(here is the root of the root and the bud of the bud
and the sky of the sky of a tree called life;which grows
higher than soul can hope or mind can hide)
and this is the wonder that's keeping the stars apart

i carry your heart(i carry it in my heart)
</pre>
```

[`PrepareRelease`]: /reference/config-file/steps/prepare-release
[packages]: /reference/concepts/package
[default workflows]: /reference/default-workflows
