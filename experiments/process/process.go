// build
package main

import (
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	//"syscall"
)

func main() {

	dateCmd := exec.Command("date")

	dateOut, err := dateCmd.Output()
	if err != nil {
		panic(err)
	}
	fmt.Println("> date")
	fmt.Println(string(dateOut))

	grepCmd := exec.Command("grep", "hello")

	grepIn, _ := grepCmd.StdinPipe()
	grepOut, _ := grepCmd.StdoutPipe()
	grepCmd.Start()
	grepIn.Write([]byte("hello grep\ngoodbye grep"))
	grepIn.Close()
	grepBytes, _ := io.ReadAll(grepOut)
	grepCmd.Wait()

	fmt.Println("> grep hello")
	fmt.Println(string(grepBytes))

	lsCmd := exec.Command("bash", "-c", "ls -a -l -h")
	lsOut, err := lsCmd.Output()
	if err != nil {
		panic(err)
	}
	fmt.Println("> ls -a -l -h")
	fmt.Println(string(lsOut))

	cmd := exec.Command("sh")
	/*cmd.SysProcAttr = &syscall.SysProcAttr{
			//Isolate UTS, IPC, PID, mount, user, network
			Cloneflags: syscall.CLONE_NEWUTS |
					syscall.CLONE_NEWIPC |
					syscall.CLONE_NEWPID |
					syscall.CLONE_NEWNS |
					syscall.CLONE_NEWUSER |
					syscall.CLONE_NEWNET,
			//Set the uid and GID of the container
			UidMappings: []syscall.SysProcIDMap{
					{
							//The uid of the container
							ContainerID: 1,
							//Uid of host
							HostID: 0,
							Size:   1,
					},
			},
			GidMappings: []syscall.SysProcIDMap{
					{
							//GID of container
							ContainerID: 1,
							//GID of host
							HostID: 0,
							Size:   1,
					},
			},
	}*/
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
			log.Fatal(err)
	}

	cmd2 := exec.Command("code", ".")

	err = cmd2.Run()
	
	if err != nil {
		log.Fatal(err)
	}
}
