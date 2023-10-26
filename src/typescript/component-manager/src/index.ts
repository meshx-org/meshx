import { Channel } from "@meshx-org/fiber-ts"
/*
export function main(arg: Channel) {
    // Initialize the outgoing services provided by this component
    let fs = ServiceFs.new_local();
    fs.dir("svc").add_fidl_service(IncomingService.Echo);
    
    // Serve the outgoing services
    fs.take_and_serve_directory_handle();
    
     // Listen for incoming requests to connect to Echo, and call run_echo_server
    // on each one
    console.log("Listening for incoming connections...")
    const MAX_CONCURRENT = 10000;
    
    await fs.for_each_concurrent(MAX_CONCURRENT, IncomingService:: Echo(stream) {
        run_echo_server(stream).unwrap_or_else((e) => console.log(e))
    })

    return 0
    
     
}
*/