using System.Collections.Generic;
using System.Text;
using System.Threading;
using System.Runtime.InteropServices;

namespace Ffi{

    public class Program
    {
        public static Dictionary<int, RustProbe> ProbeDict = new Dictionary<int, RustProbe>();

        public static void Main(String[] args)
        {
            RustProbe p1 = new RustProbe(1, "https://example.com", 5);
            RustProbe p2 = new RustProbe(2, "https://bing.com", 30);

            ProbeDict[1] = p1;
            ProbeDict[2] = p2;

            p1.StartProbing();
            p2.StartProbing();

            // Sleep 60 seconds the change to 10 second probes for p1
            Thread.Sleep(60000);
            p1.UpdateProbeConfig(10);

            // TODO stop probing p1
            Thread.Sleep(60000);
            p1.StopProbing();
            ProbeDict.Remove(1);


            Console.ReadKey();
        }

        // Using static funcion because i'm not sure passing in object's function will be good idea with garbage collection
        // I suspect .NET garbage collector will not know that the rust code still holds the function pointer to a given object.
        public static void ReportProbe(int id, bool success) {
            RustProbe? p = null;
            ProbeDict.TryGetValue(id, out p);
            if (p == null) {
                Console.WriteLine("Probe result given for non-tracked probe");
            }
            else {
                p.OnProbeResult(success);
            }
        }
    }

    public class RustProbe
    {
        public delegate void OnProbeResultRustCallback(int id, bool success);

        [DllImport("../ffi_rust/target/debug/libffi_rust.so", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern void configure_probe(byte[] url, Int32 interval, Int32 id, OnProbeResultRustCallback callback);

        [DllImport("../ffi_rust/target/debug/libffi_rust.so", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern void stop_probe(byte[] url);

        // Many more configs, different per probe type. This is probably an abstract class.
        private int id; // probably a guid passed as a string over ffi but i got lazy
        private string url;
        private int interval;

        public RustProbe(int id, string url, int interval) 
        {
            this.id = id;
            this.url = url;
            this.interval = interval;
        }

        public void StartProbing()
        {
            OnProbeResultRustCallback callback = Program.ReportProbe;
            configure_probe(Encoding.UTF8.GetBytes(this.url), this.interval, this.id, callback);
        }

        public void StopProbing()
        {
            stop_probe(Encoding.UTF8.GetBytes(this.url));
        }

        public void UpdateProbeConfig(int interval)
        {
            this.interval = interval;
            OnProbeResultRustCallback callback = Program.ReportProbe;
            configure_probe(Encoding.UTF8.GetBytes(this.url), this.interval, this.id, callback);
        }

        public void OnProbeResult(bool success) 
        {
            Console.WriteLine($"Probe result: id: {id}, url: {url}, success: {success}");
        }
    }

}
