# web-project-template

_A project template for full-stack web projects with
[Rust](https://www.rust-lang.org/) and [NextJS](https://nextjs.org/)._

### Setup

1. Clone the repo:

   ```bash
   git clone git@github.com:hulloitskai/web-project-template
   ```

2. Bootstrap the workspace:

   ```bash
   ./bootstrap-workspace.sh
   ```

3. Fill out [`.env`](.env) file:

   ```bash
   vi .env
   ```

4. Start background dependencies:

   ```bash
   docker compose up -d
   ```

5. Run database migrations:

   ```bash
   cd migrator && yarn up
   ```

6. In **Terminal 1**, start `api`:

   ```bash
   cd api && cargo run
   ```

7. In **Terminal 2**, start `web`:

   ```bash
   cd web && yarn dev
   ```

### Teardown

1. Close both **Terminal 1** and **Terminal 2**.

2. Stop background dependencies:

   ```bash
   docker compose down
   ```
