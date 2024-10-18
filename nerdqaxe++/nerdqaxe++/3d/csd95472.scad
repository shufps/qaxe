$fn=50;

color("#505050") translate([0,0,0.5]) cube([5,6,1], center=true);

color("lightgray") union() {
    for (i=[-5:5]) {
        translate([5/2-0.24, i*0.5, 0.095]) cube([0.5, 0.25, 0.2], center=true);
        translate([-5/2+0.24, i*0.5, 0.095]) cube([0.5, 0.25, 0.2], center=true);
    }
    
    translate([1+0.25/2, 3-0.10, 0.1]) cube([0.25, 0.3, 0.25], center=true);
    translate([-(1+0.25/2), 3-0.10, 0.1]) cube([0.25, 0.3, 0.25], center=true);

    translate([1+0.25/2, -(3-0.10), 0.1]) cube([0.25, 0.3, 0.25], center=true);
    translate([-(1+0.25/2), -(3-0.10), 0.1]) cube([0.25, 0.3, 0.25], center=true);

    translate([0, 0, 0.1]) cube([3, 5.5, 0.25], center=true);    
}


