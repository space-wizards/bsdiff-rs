using System;
using System.IO;
using bsdiff.net;

namespace test
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello World!");
            string a = Directory.GetCurrentDirectory();
            Console.WriteLine(a);

            Console.WriteLine("oh snap: {0}", Class1.Derp());
            Console.ReadLine();
        }
    }
}
