

# Generating the ABI Bindings

To generate them, we used the following command:

```bash
bindgen --use-core \
        --allowlist-function '[A-Z][A-Z_]+' \
        --allowlist-type '[A-Z][A-Z_]+' \
        --allowlist-var '[A-Z][A-Z_]+' \
        --blocklist-type '_.*' \
        --ctypes-prefix cty wrapper.h
```

with the following `wrapper.h`:

```c
#include <apex_blackboard.h>
#include <apex_buffer.h>
#include <apex_error.h>
#include <apex_event.h>
#include <apex_irq_event_np.h>
#include <apex_multiple_schedules.h>
#include <apex_partition.h>
#include <apex_partition_np.h>
#include <apex_process.h>
#include <apex_queuing.h>
#include <apex_sampling.h>
#include <apex_semaphore.h>
#include <apex_system_np.h>
#include <apex_time.h>
#include <apex_types.h>
```
