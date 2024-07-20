#!/bin/sh
cd ~/Documents/SWP/aqua-PKC \
&& echo "starting PKC" \
&& yes | ./pkc nuke \
&& ./pkc setup -w 0xd0fFc39Fb1968864E386b888D2b1e4e34fF65393 \
# ./pkc start \
sleep 5 \
&& cd ~/Documents/SWP/aqua-guardian \
&& echo "running tests" \
./testing.sh \
&& echo "Tests were successsfully completed" || echo "<- these did not work. For reasons see above." \
&& echo "Testing CLI utility"\
&& cargo run -q --bin guardian-api-cli -- start-server \
&& echo Yay || echo broken \
&& cd ~/Documents/SWP/aqua-PKC \
&& ./pkc stop \
&& echo "PKC stopped" \