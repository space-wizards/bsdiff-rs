using System;
using System.Runtime.InteropServices;

namespace Bsdiff
{
    public static class Patch
    {
        [DllImport("bsdiffwrapper.dll", EntryPoint = "derp")]
        public static extern int Derp();
    }
}
