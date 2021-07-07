<?php

class Field{
	public $walls = array("left"=>false, "top"=>false, "right"=>false, "bottom"=>false);
	
	//if goal isn't null then it is an array with two 
	public $goal = NULL;   
	//robots will be represented with colors 
	public $robot = NULL; 

	function __construct($walls, $goal = NULL, $robot = NULL){
		if(strlen($walls) !== 4){
			echo "Eror while making field.";
			exit;
		}
		if($walls[0] === "1"){
			$this->walls["left"] = true;
		}

		if($walls[1] === "1")
			$this->walls["top"] = true;
		
		if($walls[2] === "1")
			$this->walls["right"] = true;
		
		if($walls[3] === "1")
			$this->walls["bottom"] = true;
		
		$this->goal = $goal;
		$this->robot = $robot;
	}

	function draw_field(){
		echo "<td class=\"board_field\" style = \"";
		
		// ovo je za polja u sredini
		$middle_field = false;
		if($this->robot === BLACK){
			echo "background: black; ";
			$middle_field = true;
		}
		//treba dodati i za one ciljeve ali o tom po tom

		foreach($this->walls as $wall=>$exists)
			if ($exists)
				echo "border-" . $wall . ": 5px solid brown;";

		echo "\">";
		if(! $middle_field ){
			if($this->robot == NULL && $this->goal == NULL){
				echo "<i class=\"fas fa-football-ball \"></i>";
			}
			else{
				if($this->goal != NULL){
					echo "<span class=\"fa-stack\">";
					
					if($this->robot != NULL) echo "<i class=\"fas fa-robot fa-stack-1x\" style = \"background-color:".$this->robot."\"></i>";
					else echo "<i class=\"fa-stack-1x ".$this->goal[1]."\" style = \"color:".$this->goal[0]."\"></i>";

					echo "</span>";
				}
				else{
					echo "<span class=\"fa-stack\">";
					echo "<i class=\"fas fa-robot fa-stack-1x\" style = \"background-color:".$this->robot."\"></i>";
					echo "</span>";
				}
			}
			echo "</td>";
		}
	}

};
?>