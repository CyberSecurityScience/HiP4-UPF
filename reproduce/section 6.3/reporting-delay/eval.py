
import numpy as np
import glob
import sys
from natsort import natsorted

for f in natsorted(glob.glob(sys.argv[1])) :
    diff_ref = []
    num_non_report = 0
    num_neg = 0
    with open(f, 'r', encoding='utf-8') as fp :
        for l in fp :
            do_break = False
            l = l.strip()
            if l :
                try :
                    [_, a, b, _, _] = l.split(',')
                    if a == '-1' or b == '-1' :
                        num_non_report += 1
                        continue
                        diff_ref.append(1000000000000000000)
                    else :
                        diff_ref.append(int(b) - int(a))
                except Exception :
                    do_break = True
            if do_break: break
    try :
        ref = np.array(diff_ref, dtype=np.float64) / 1000
        ref_p99 = np.percentile(ref, 99)
        ref_mean = np.mean(ref)
        ref_std = np.std(ref)
        print(f, '\t', ref_mean,  '\t', ref_std, '\t', ref_p99)
    except Exception :
        continue

