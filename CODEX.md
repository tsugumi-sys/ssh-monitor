## Overview

- tui: UI modules. Fetch data from db (states), and render them.
- backend: Backend modules. Execute commands via ssh and get metrics, save into sqlite.

## TUI

The modules are separated by screens.

Each screen has the following basic files.

- view: main render function.
- update: main update states method.
- (we can add states, to define and handle states)


And we extract sub components or methods into the other files.

## Backend

- backend/db: Create database client, initialize database.
- backend/ssh: Core ssh connection handlers.
- backend/jobs: All jobs to fetch metrics via ssh. We define each jobs per metrics, and execute them in a certaing group to reduce the ssh connections.
