package main

/*import (
	"go.uber.org/zap"
)*/

/*func main() {
	logger, _ := zap.NewDevelopment()
	zap.ReplaceGlobals(logger)
	zap.L().Info("Hello World!")
}*/

import (
	"crypto/sha1"
	"fmt"
	"hash"
	"io"
	"net"

	"context"

	"capnproto.org/go/capnp/v3/rpc"
	"capnproto.org/go/capnp/v3/server"
	"github.com/meshx-org/meshx/cmd/meshx/hashes"
	//"golang.org/x/net/context"
)

type listenCall struct {
	c   net.Conn
	err error
}

func tcpPipe() (t1, t2 net.Conn, err error) {
	host, err := net.LookupIP("localhost")
	if err != nil {
		return nil, nil, err
	}
	l, err := net.ListenTCP("tcp", &net.TCPAddr{IP: host[0], Port: 32451})
	if err != nil {
		return nil, nil, err
	}
	ch := make(chan listenCall)
	abort := make(chan struct{})
	go func() {
		c, err := l.AcceptTCP()
		select {
		case ch <- listenCall{c, err}:
		case <-abort:
			c.Close()
		}
	}()
	laddr := l.Addr().(*net.TCPAddr)
	c2, err := net.DialTCP("tcp", nil, laddr)
	if err != nil {
		close(abort)
		l.Close()
		return nil, nil, err
	}
	lc := <-ch
	if lc.err != nil {
		c2.Close()
		l.Close()
		return nil, nil, err
	}
	return lc.c, c2, nil
}

var policy = server.Policy{}

// hashFactory is a local implementation of HashFactory.
type hashFactory struct{}

func (hf hashFactory) NewSha1(_ context.Context, call hashes.HashFactory_newSha1) error {
	// Create a new locally implemented Hash capability.
	hs := hashes.Hash_ServerToClient(hashServer{sha1.New()}, &policy)
	// Notice that methods can return other interfaces.
	res, _ := call.AllocResults()

	return res.SetHash(hs)
}

// hashServer is a local implementation of Hash.
type hashServer struct {
	h hash.Hash
}

func (hs hashServer) Write(_ context.Context, call hashes.Hash_write) error {
	data, err := call.Args().Data()
	if err != nil {
		return err
	}

	_, err = hs.h.Write(data)
	return err
}

func (hs hashServer) Sum(_ context.Context, call hashes.Hash_sum) error {
	res, err := call.AllocResults()
	if err != nil {
		return err
	}

	b := hs.h.Sum(nil)
	return res.SetHash(b)
}

func serveHash(ctx context.Context, rwc io.ReadWriteCloser) error {
	// Create a new locally implemented HashFactory.
	main := hashes.HashFactory_ServerToClient(hashFactory{}, &policy)

	// Listen for calls, using the HashFactory as the bootstrap interface.
	conn := rpc.NewConn(rpc.NewStreamTransport(rwc), &rpc.Options{
		BootstrapClient: main.Client,
	})
	defer conn.Close()

	// Wait for connection to abort.
	select {
	case <-conn.Done():
		return nil
	case <-ctx.Done():
		return conn.Close()
	}
}

func client(ctx context.Context, rwc io.ReadWriteCloser) error {
	// Create a connection that we can use to get the HashFactory.
	conn := rpc.NewConn(rpc.NewStreamTransport(rwc), nil) // nil sets default options
	defer conn.Close()

	// Get the "bootstrap" interface.  This is the capability set with
	// rpc.MainInterface on the remote side.
	hf := hashes.HashFactory{Client: conn.Bootstrap(ctx)}

	// Now we can call methods on hf, and they will be sent over c.
	// The NewSha1 method does not have any parameters we can set, so we
	// pass a nil function.
	f1, free := hf.NewSha1(ctx, nil)
	defer free()

	// 'NewSha1' returns a future, which allows us to pipeline calls to
	// returned values before they are actually delivered.  Here, we issue
	// calls to an as-of-yet-unresolved Sha1 instance.
	sha := f1.Hash()

	// s refers to a remote Hash.  Method calls are delivered in order.
	_, free = sha.Write(ctx, func(p hashes.Hash_write_Params) error {
		err := p.SetData([]byte("Hello, "))
		fmt.Println(p)
		return err
	})
	defer free()

	_, free = sha.Write(ctx, func(p hashes.Hash_write_Params) error {
		err := p.SetData([]byte("World!"))

		fmt.Println(p)
		return err
	})
	defer free()

	// Get the sum, waiting for the result.
	sum, free := sha.Sum(ctx, nil)
	defer free()

	fmt.Println(sum)

	result, err := sum.Struct()
	if err != nil {
		return err
	}

	// Display the result.
	sha1Val, err := result.Hash()
	if err != nil {
		return err
	}

	fmt.Printf("sha1: %x\n", sha1Val)
	return nil
}

func main() {

	ctx := context.Background()

	//c1, c2, err := tcpPipe()

	c1, c2 := BufferedPipe()

	//if err != nil {
	//	fmt.Println(err)
	//	os.Exit(1)
	//}

	go serveHash(ctx, c2)
	client(ctx, c1)
}
