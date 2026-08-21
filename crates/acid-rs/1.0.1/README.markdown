<p align="center">
 <img src="https://github.com/iamtheblackunicorn/acid/raw/main/assets/images/banner.png"/>
</p>

# ACID

***A supersonic static-site generator written in Rust.***

![GitHub CI](https://github.com/iamtheblackunicorn/acid/actions/workflows/rust.yml/badge.svg)

## ABOUT

I've been an avid user of the Jekyll CMS for many years now and have used this CMS to build all of my websites with. However, I recently discovered that Ruby is on a downward trend in the developer community, and since I was also looking to improve my Rust skills, I wondered how one would write a blog using Rust. This is when I realized that there aren't many options. ***Acid** is another option. ***Acid*** works quite similarly to Jekyll but is a bit more bare-bones at this point. (This may change.) Enjoy. :)

## LINKS

- ***Acid***'s page on [Crates.io](https://crates.io): [View](https://crates.io/crates/acid-rs).
- A live, deployed Acid project: [View](https://blckunicorn.art/acid).
- A quick getting-started guide on my mixed blog: [Read](https://angeldustduke.art/2022/03/18/How-to-write-a-blog-in-Rust.html).
- The same guide on Hashnode (Likes, Follows, and comments are always appreciated.): [Read](https://angeldustduke.hashnode.dev/how-to-create-a-supersonic-blog-in-a-flash).

## BUILDING

### Tools

You will need the following tools installed and available:

- Rust
- Git

### Steps

- 1.) Get the source code:
```bash
$ git clone https://github.com/iamtheblackunicorn/acid.git
```
- 2.) Change directory:
```bash
$ cd acid
```
- 3.) Build the source code:
```bash
$ cargo build --release
```

## INSTALLATION

### Requirements

You will need the Rust toolchain and Git installed and available from the command line. Once that is done, you can install ***Acid*** with the following commands. These commands will work on all platforms.

### Installation

- Install the latest cutting-edge version of ***Acid***.

```bash
$ cargo install --git https://github.com/iamtheblackunicorn/acid.git
```

- Install the latest stable release directly from [Crates.io](https://crates.io/creates/acid-rs).

```bash
$ cargo install acid-rs
```

## USAGE

### Command Line usage

- To compile your project, simply run this command on the command line:

```bash
$ acid build yourprojectdir
```

`yourprojectdir` represents the path of your project.

- If you would like to clean up your project, simply run this command on the command line:

```bash
$ acid clean yourprojectdir
```
This will delete the `build` directory with your compiled project inside your project directory called `yourprojectdir`.

### Creating a new project.

Creating a new ***Acid*** site entails the following steps. ***Acid*** is very modular, allowing ***YOU*** to extend your site as you see fit. There are some basic steps, however. These are the steps you need to take to create a new ***Acid*** site. My recommendation is that you have a look at the `site` directory in this repository.

- 1.) Create a new directory.
- 2.) Inside this directory, create a new file called `config.json`. `config.json` could have the following contents:
  ```json
  {
    "title":"ACID.RS",
    "baseurl":"/acid/",
    "has_assets":"true",
    "assets_path":"assets",
    "description":"A supersonic static-site generator written in Rust."
  }
  ```
  - `title`: The title of your site.
  - `baseurl`: The "root" URL of your site. If you're deploying your site on GitHub Pages as an apex site, this field can be filled with `/`. If not, fill this with `/your_repo/`.
  - `has_assets`: Does your site have local static assets? These are copied to your build folder at compile time.
  - `assets_path`: Where inside your site folder are these assets?
  - `description`: Your site's description.
  - Further fields: You can fill your `config.json` with as many fields as you like. All fields will be available via the `{{ site.field }}` variable in your templates, where `field` is a placeholder for any other field you might have.

- 3.) Inside this directory, create a new called `index.markdown`. This file is the base for generating your site's     `index.html` file. This file could look something like this:
  ```markdown
  ---
  layout:blog
  title:ACID.RS
  ---
  ```
  - `layout`: This YAML field tells ***Acid*** which layout to build your `index.html` from.
  - `title`: This YAML field tells ***Acid*** the page title.
  - Further fields: You can fill your Markdown files with as many fields as you like. All fields will be available via the `{{ page.field }}` variable in your templates, where `field` is a placeholder for any other field you might have.

- 4.) Inside the same directory, create three new directories: `layouts`, `posts`, `pages`, and `assets`.
  - `layouts`: This directory holds all your site's templates. These templates are Liquid templates. A sample layout for the main blog overview page, called `blog.html`, could look something like this:
    ```Liquid
    <!DOCTYPE html>
    <html>
    <head>
    <link rel="stylesheet" href="{{ 'assets/css/styles.css' | prepend: site.baseurl }}"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="{{ site.description }}"/>
    <title>{{ site.title }}</title>
    </head>
    <body>
    <h1>{{ site.title }}</h1>
    <p class="subtitle">{{ site.description }}</p>
    {% for post in posts %}
    <div class="content">
    <h2>{{ post.title }}</h2>
    <p>{{ post.description }}</p>
    <p><a href="{{ post.url }}">READ ME</a></p>
    </div>
    {% endfor %}
    <div class="footer">
    <p class="footer">Proudly hosted by GitHub and powered by <a href="https://github.com/iamtheblackunicorn/acid">ACID.RS</a></p>.
    </div>
    </body>
    </html>
    ```
    - Please note that your site's posts are stored in a dictionary called `posts`. Each post can be accessed via a `for` loop.
    - A template for a page or post, called `page.html` or `post.html` could look something like this:
    ```Liquid
    <!DOCTYPE html>
    <html>
    <head>
    <link rel="stylesheet" href="{{ 'assets/css/styles.css' | prepend: site.baseurl }}"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="{{ site.description }}"/>
    <title>{{ site.title }}</title>
    </head>
    <body>
    <h1>{{ site.title }}</h1>
    <p class="subtitle">{{ site.description }}</p>
    <div class="content">
    {{ page.content }}
    </div>
    <div class="footer">
    <p class="footer">Proudly hosted by GitHub and powered by <a href="https://github.com/iamtheblackunicorn/acid">ACID.RS</a></p>.
    </div>
    </body>
    </html>
    ```
  - `posts`: This directory contains your site's posts. These are Markdown files with filenames like this: `YYYY-MM-DD-Your-Title.markdown`. `YYYY-MM-DD` represents your post's date.
    - A sample post file by the name of `2020-03-09-Welcome.markdown` could look something like this:
    ```markdown
    ---
    layout:post
    title:Release Notes v.1.0.0
    description:Notes on the v.1.0.0. release.
    ---
    ## Version 1.0.0
    This is the first version of Acid, version 1.0.0.
    ## Changes
    - Initial release.
    - Initial upload to GitHub.
    ```
    - The same rules for `index.markdown` apply here with the difference that you can write your content here.
    - `pages`: This directory contains content pages, this could be an `about` page for example.
    - A sample content page called `about.makdown` could look something like this:
    ```markdown
    ---
    layout:page
    title:About
    ---
    ## About
    This is a sample about page. Write something about yourself here.
    ```
    - The same rules for `index.markdown` apply here with the difference that you can write your content here.
  - `assets`: This directory contains your site's static assets, like CSS and Javascript.
- 5.) Once everything is in place, you change directory into your site's directory and run the following command:
  ```bash
  $ acid build .
  ```
  - Once the build succeeds, your shiny new site will be available in a sub-directory called `build`.

### Deployment to GitHub Pages.

If you have a GitHub account, you can upload your project to a repository, create a new branch called `gh-pages`, create a new file called `rust.yml` at `.github/workflows` in your repository, fill it with the code below, and voilá: You can now view your project on the web under the URL of `yourusername.github.io/yourporject`.

```YAML
on: [push]
name: Acid Project CI
jobs:
  build_and_test:
    name: Acid Project CI
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: actions-rs/cargo@v1
        with:
          command: build
          args: --release
      - uses: actions-rs/cargo@v1
        with:
          command: run
          args: build .
      - name: Deploy
        uses: JamesIves/github-pages-deploy-action@v4.2.5
        with:
          branch: gh-pages
          folder: build
```

## CONTRIBUTING

If you have some suggestions for improvement or you want to contribute, either file an issue or fork the repository. If you want to do the latter, make and test your changes, and file a Pull Request.

## CHANGELOG

### Version 1.0.0

- Initial release.
- Upload to GitHub.

### Version 1.0.1

- Minor fixes.
- Fixed some typos and dead links.
- Updated the documentation.

## NOTE

- *Acid* by Alexander Abraham :black_heart: a.k.a. *"The Black Unicorn" :unicorn:*
- Licensed under the MIT license.
