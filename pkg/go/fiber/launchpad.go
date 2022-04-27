package fiber

// API OVERVIEW
// -------------------------------------------------------------
//
// Launchpad is designed to be used like this:
//   launchpad_t* lp;
//   launchpad_create(job, "processname", &lp);
//   launchpad_load_from_file(lp, argv[0]);
//   launchpad_set_args(lp, argc, argv);
//   launchpad_set_environ(lp, env);
//   << other launchpad_*() calls to setup initial fds, handles, etc >>
//   zx_handle_t proc;
//   const char* errmsg;
//   zx_status_t status = launchpad_go(lp, &proc, &errmsg);
//   if (status < 0)
//       printf("launchpad failed: %s: %d\n", errmsg, status);
//
// If any of the calls leading up to launchpad_go(), including
// launchpad_create() itself fail, launchpad_go() will return
// an error, and (if errmsg is non-NULL) provide a human-readable
// descriptive string.
// If proc is NULL, the process handle is closed for you.
//
// There are alternative versions of launchpad_create_*() which
// provide more options, various simple and complex alternatives to
// launchpad_load_*(), and a variety of functions to configure fds,
// handles, etc, which are passed to the new process.  They are
// described in detail below.

// CREATION: one launchpad_create*() below must be called to create
// a launchpad before any other operations may be one with it.
// ----------------------------------------------------------------

// Create a new process and a launchpad that will set it up.
// The job handle is used for creation of the process, but is not
// taken ownership of or closed.
//
// If the job handle is 0 (ZX_HANDLE_INVALID), the default job for
// the running process is used, if it exists (zx_job_default()).
//
// The job used will be cloned and given to the new process.

// TODO: zx_status_t launchpad_create(zx_handle_t job, const char* name, launchpad_t** lp);

// Create a new process and a launchpad that will set it up.
// The creation_job handle is used to create the process but is
// not taken ownership of or closed.
//
// The transferred_job handle is optional.  If non-zero, it is
// consumed by the launchpad and will be passed to the new process
// on successful launch or closed on failure.

// TODO: zx_status_t launchpad_create_with_jobs(zx_handle_t creation_job, zx_handle_t transferred_job,
//                                       const char* name, launchpad_t** result);
